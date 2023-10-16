use std::collections::HashMap;
use std::time::Instant;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use flume::unbounded;
use log::{debug, error, trace, warn};
use snowflake::SnowflakeIdBucket;

use crate::conductor_module::def::ConductorModule;
use crate::conductor_module::errors::{
    HandleLoginError, ProcessGameEventError, ProcessModuleEventError, SendEventToModuleError,
};
use crate::core::blueprint::BlueprintService;
use crate::core::guest::{Admin, Guest, LoginData, ModuleEnterSlot, ProviderUserId};
use crate::core::module::{
    AdminToSystemEvent, CommunicationEvent, EnterFailedState, EnterSuccessState, GamePosition,
    GameSystemToGuest, GuestEvent, GuestStateChange, GuestTo, GuestToModule, GuestToModuleEvent,
    GuestToSystemEvent, LeaveFailedState, LeaveSuccessState, ModuleIO, ModuleName, ModuleState,
    ModuleToSystem, ModuleToSystemEvent, SignalToGuest, SystemCommunicationIO, SystemToModuleEvent,
    ToastAlertLevel,
};
use crate::core::module_system::DynamicGameModule;
use crate::core::{blueprint, send_and_log_error};
use crate::core::{fix_intellij_error_bug, safe_unwrap, Snowflake, LOGGED_IN_TODAY_DELAY_IN_HOURS};
use crate::login::login_manager::{LoginError, LoginManager};
use crate::persistence_module::models::{PersistedGuest, UpdatePersistedGuestState};
use crate::persistence_module::{PersistenceError, PersistenceModule};
use crate::resource_module::def::GuestId;
use crate::webserver_module::def::WebServerModule;
use crate::{ResourceModule, SystemModule, WebsocketModule};

impl SystemModule for ConductorModule {
    fn module_name(&self) -> ModuleName {
        String::from("ConductorModule")
    }

    fn status(&self) -> &ModuleState {
        todo!()
    }

    fn start(&mut self) {
        todo!()
    }

    fn shutdown(&mut self) {
        todo!()
    }
}

impl ConductorModule {
    pub async fn conduct(&mut self) {
        self.handle_new_ws_connections();
        self.handle_lost_ws_connections();
        self.move_guests();
        self.update_modules();

        self.process_events_from_modules();
        self.process_events_from_guest();
        self.send_system_events_to_guests();
        self.process_logins();
        self.send_load_events();

        self.handle_timeouts();

        self.handle_admin_events().await;
    }

    #[cfg(feature = "local")]
    async fn check_admin_login(_admin: &Admin) -> bool {
        true
    }

    #[cfg(not(feature = "local"))]
    async fn check_admin_login(_admin: &Admin) -> bool {
        false
    }

    async fn handle_admin_events(&mut self) {
        for (ws_connection_id, admin) in &mut self.admins {
            for message in self.websocket_module.drain_events(ws_connection_id) {
                match serde_json::from_str::<AdminToSystemEvent>(message.to_string().as_str()) {
                    Ok(event) => {
                        if !admin.is_logged_in {
                            if let AdminToSystemEvent::ProviderLoggedIn(provider) = event {
                                if Self::check_admin_login(admin).await {
                                    debug!("Admin login successful!");
                                    admin.is_logged_in = true;
                                } else {
                                    error!("Admin login failed!");
                                }
                            } else {
                                error!("Admin tried to do something other than logging in while not being logged in!");
                            }
                            continue;
                        }

                        match event {
                            AdminToSystemEvent::SetMainDoorStatus(status) => {
                                debug!("Setting main door status");
                                self.web_server_module.set_main_door_status(status).await;
                            }
                            AdminToSystemEvent::SetBackDoorStatus(status) => {
                                debug!("Setting back door status");
                                self.web_server_module.set_back_door_status(status).await;
                            }
                            AdminToSystemEvent::ProviderLoggedIn(_) => {
                                error!("Admin should already be logged in!")
                            }
                            AdminToSystemEvent::UpdateConductor(conductor) => {
                                if let Err(err) =
                                    self.blueprint_service.save_conductor_blueprint(&conductor)
                                {
                                    error!("Could not save conductor blueprint! {:?}", err)
                                }
                            }
                            AdminToSystemEvent::UpdateModule(module_name, module_update) => {}
                            AdminToSystemEvent::CreateModule(module_name) => {
                                Self::lazy_load_module(
                                    module_name,
                                    &mut self.module_map,
                                    &mut self.module_communication_map,
                                    &mut self.resource_module,
                                    &self.blueprint_service,
                                );
                            }
                            AdminToSystemEvent::DeleteModule(module_name) => {}
                        }
                    }
                    Err(err) => error!("Failed to parse admin event! {:?}", err),
                }
            }
        }
    }

    fn lazy_load_module(
        module_name: ModuleName,
        module_map: &mut HashMap<ModuleName, DynamicGameModule>,
        module_communication_map: &mut HashMap<ModuleName, ModuleIO>,
        resource_module: &mut ResourceModule,
        blueprint_service: &BlueprintService,
    ) {
        match DynamicGameModule::lazy_load(module_name.clone(), blueprint_service, resource_module)
        {
            Ok((dynamic_game_module, module_output_receiver, module_input_sender)) => {
                module_map.insert(module_name.clone(), dynamic_game_module);
                module_communication_map.insert(
                    module_name.clone(),
                    ModuleIO {
                        receiver: module_output_receiver,
                        sender: module_input_sender,
                    },
                );
            }
            Err(err) => error!("Could not create dynamic module: {:?}", err),
        }
    }

    fn handle_timeouts(&mut self) {
        for (guest_id, connection_lost_time) in &self.guest_timeout_map {
            if connection_lost_time.elapsed().as_secs() > 30 {
                self.timeouts.push(*guest_id);
            }
        }
        for guest_id in self.timeouts.drain(..) {
            if let Some(guest) = self.guests.get(&guest_id) {
                if let Some(module_name) = &guest.current_module {
                    if let Some(module) = self.module_map.get_mut(module_name) {
                        if let Err(err) = module.try_leave(guest) {
                            error!("Guest could not leave module on timeout, reason: {:?}", err);
                        }
                    }
                }
                if let Some(login_data) = &guest.login_data {
                    self.provider_id_to_guest_map
                        .remove(&login_data.provider_user_id);
                }
                self.guest_timeout_map.remove(&guest_id);
                self.session_id_to_guest_map.remove(&guest.session_id);
            }
            self.guests.remove(&guest_id);
            debug!("Guest removed {}.", guest_id);
        }
    }

    pub fn update_modules(&mut self) {
        for module in self.module_map.values_mut() {
            module.update();
        }
    }

    pub fn new(
        websocket_module: WebsocketModule,
        blueprint_service: BlueprintService,
        blueprint: blueprint::Conductor,
    ) -> ConductorModule {
        let snowflake_gen = SnowflakeIdBucket::new(1, 1);
        let mut module_communication_map = HashMap::new();
        let mut module_map = HashMap::new();
        let mut resource_module = ResourceModule::new();
        let (sender, receiver) = unbounded();

        Self::lazy_load_module(
            "Dummy".into(),
            &mut module_map,
            &mut module_communication_map,
            &mut resource_module,
            &blueprint_service,
        );

        ConductorModule {
            blueprint,
            blueprint_service,
            websocket_module,
            resource_module,
            persistence_module: PersistenceModule::new(),
            web_server_module: WebServerModule::new(),
            login_manager: LoginManager::new(),
            snowflake_gen,
            module_connection_map: HashMap::from([(
                "".into(),
                ("Dummy".into(), "DummyEnter".into()),
            )]),
            guests: HashMap::new(),
            admins: HashMap::new(),

            ws_to_guest_map: HashMap::new(),
            provider_id_to_guest_map: HashMap::new(),
            provider_id_to_admin_map: HashMap::new(),

            session_id_to_guest_map: HashMap::new(),
            session_id_to_admin_map: HashMap::new(),
            guest_timeout_map: HashMap::new(),

            timeouts: Vec::new(),
            module_map,

            module_communication_map,
            system_to_guest_communication: SystemCommunicationIO { receiver, sender },
        }
    }

    pub fn handle_new_ws_connections(&mut self) {
        for (connection_id, ticket) in self.websocket_module.handle_new_ws_connections() {
            debug!("{:?}", ticket);
            if let Some(true) = ticket.admin_login {
                debug!("Admin ready to start their session!");
                self.admins.insert(
                    connection_id,
                    Admin {
                        id: self.snowflake_gen.get_id(),
                        login_data: None,
                        is_logged_in: false,
                        ws_connection_id: connection_id,
                    },
                );

                continue;
            }

            let guest_id_from_session_id = self
                .session_id_to_guest_map
                .get(&ticket.session_id.unwrap_or_default())
                .unwrap_or(&0);

            let guest_id: Snowflake =
                if let Some(guest) = self.guests.get_mut(guest_id_from_session_id) {
                    debug!("Guest already existed with their session!");
                    if guest.ws_connection_id.is_some() {
                        error!("Guest already has a connection!");
                        //TODO: Disconnect old connection and connect new connection
                        self.websocket_module.send_event(
                            &connection_id,
                            serde_json::to_string(&CommunicationEvent::AlreadyConnected)
                                .unwrap_or_else(|_| "AlreadyConnected".to_string()),
                        );
                        continue;
                    }
                    self.guest_timeout_map.remove(&guest.id);
                    self.ws_to_guest_map.insert(connection_id, guest.id);

                    guest.ws_connection_id = Some(connection_id);

                    if let Some(current_module) = &guest.current_module {
                        if let Some(module_communication) =
                            self.module_communication_map.get_mut(current_module)
                        {
                            if let Err(err) = module_communication
                                .sender
                                .system_to_module_sender
                                .send(GuestEvent {
                                    guest_id: guest.id,
                                    event_type: SystemToModuleEvent::Reconnected,
                                })
                            {
                                error!("Error sending reconnect event ${}", err);
                            }

                            if let Err(err) = self
                                .resource_module
                                .activate_resources_for_guest(current_module.clone(), guest.id)
                            {
                                error!(
                                    "Error activating resource for guest {}",
                                    fix_intellij_error_bug(&err)
                                );
                            };
                        }
                    }
                    guest.id
                } else {
                    self.create_new_guest(connection_id)
                };

            if let Some(guest) = self.guests.get(&guest_id) {
                if let Ok(event_as_string) =
                    serde_json::to_string(&CommunicationEvent::ConnectionReady((
                        guest.session_id.clone(),
                        guest.login_data.is_none(),
                    )))
                {
                    Self::send_to_guest(guest, &mut self.websocket_module, event_as_string);
                } else {
                    error!("Could not parse ConnectionReady enum, wtf?");
                }
            }
        }
    }

    fn create_new_guest(&mut self, connection_id: Snowflake) -> Snowflake {
        let guest_id = self.snowflake_gen.get_id();
        let session_id = self.snowflake_gen.get_id().to_string();
        debug!("new guest {:?}", &guest_id);
        self.ws_to_guest_map.insert(connection_id, guest_id);
        self.session_id_to_guest_map
            .insert(session_id.clone(), guest_id);
        self.guests.insert(
            guest_id,
            Guest {
                id: guest_id,
                current_module: None,
                login_data: None,
                pending_module_exit: None,
                ws_connection_id: Some(connection_id),
                persisted_guest: None,
                session_id,
            },
        );
        guest_id
    }

    pub fn handle_lost_ws_connections(&mut self) {
        self.websocket_module.drop_lost_ws_connections();

        for connection_id in self.websocket_module.drain_lost_connections() {
            if let Some(admin) = self.admins.remove(&connection_id) {
                continue;
            }

            if let Some(guest_id) = self.ws_to_guest_map.remove(&connection_id) {
                if let Some(guest) = self.guests.get_mut(&guest_id) {
                    debug!("guest connection lost {:?}", &guest_id);
                    guest.ws_connection_id = None;

                    self.guest_timeout_map
                        .insert(guest_id.clone(), Instant::now());

                    if let Some(current_module) = &guest.current_module {
                        if let Some(module_communication) =
                            self.module_communication_map.get_mut(current_module)
                        {
                            if let Err(err) = module_communication
                                .sender
                                .system_to_module_sender
                                .send(GuestEvent {
                                    guest_id,
                                    event_type: SystemToModuleEvent::Disconnected,
                                })
                            {
                                error!("Could not send Disconnected event {}", err);
                            }
                        }
                    }
                } else {
                    error!(
                        "Guest {:?} for connection {:?} no longer exists?",
                        guest_id, connection_id
                    );
                }
            } else {
                warn!("connection {:?} did not have a guest assigned to it while trying to remove it.", connection_id);
            }
        }
    }

    pub fn send_load_events(&mut self) {
        for GuestEvent {
            guest_id,
            event_type,
        } in self.resource_module.drain_load_events()
        {
            if let Ok(message_as_string) =
                serde_json::to_string(&CommunicationEvent::ResourceEvent(event_type))
            {
                debug!("Sending load event to guest");
                if let Some(guest) = self.guests.get(&guest_id) {
                    Self::send_to_guest(guest, &mut self.websocket_module, message_as_string);
                }
            } else {
                error!("Error serializing resource event!");
            }
        }
    }

    pub fn send_to_guest(
        guest: &Guest,
        websocket_module: &mut WebsocketModule,
        event_as_string: String,
    ) {
        if let Some(ws_connection_id) = &guest.ws_connection_id {
            websocket_module.send_event(ws_connection_id, event_as_string);
        } else {
            trace!(
                "Could not send to guest '{}' no active connection",
                guest.id
            );
        }
    }

    pub fn move_guests(&mut self) {
        for guest in self.guests.values_mut() {
            if let Some(module_exit_slot) = &guest.pending_module_exit {
                if let Some((target_module_name, module_enter_slot)) =
                    self.module_connection_map.get(module_exit_slot)
                {
                    if let Some(current_module_name) = &guest.current_module {
                        if current_module_name == target_module_name {
                            error!("current module {} and target module {} are the same, this should never happen!", current_module_name, target_module_name);
                            continue;
                        }
                        let current_module_option = self.module_map.get_mut(current_module_name);
                        if let Some(current_module) = current_module_option {
                            ConductorModule::try_leave_module(
                                guest,
                                current_module,
                                &mut self.resource_module,
                            );
                        } else {
                            error!(
                                "Module {} did not exist, so user cannot leave it, guest is stuck!",
                                current_module_name
                            );
                        }
                    }

                    debug!(
                        "trying to get into {:?} {:?}",
                        target_module_name,
                        self.module_map.len()
                    );

                    if let Some(target_module) = self.module_map.get_mut(target_module_name) {
                        ConductorModule::try_enter_module(
                            guest,
                            module_enter_slot,
                            target_module,
                            &mut self.resource_module,
                        );
                    } else {
                        error!(
                            "Module {} did not exist, so user cannot enter it, guest is stuck!",
                            target_module_name
                        );
                    }
                } else {
                    error!(
                        "No module configured for exit_slot {} so user cannot leave!",
                        module_exit_slot
                    );
                }
            }
        }
    }

    pub fn try_leave_module(
        guest: &mut Guest,
        module: &mut DynamicGameModule,
        resource_module: &mut ResourceModule,
    ) {
        match module.try_leave(guest) {
            Ok(LeaveSuccessState::Left) => {
                guest.current_module = None;
                if let Err(err) =
                    resource_module.disable_resources_for_guest(module.name(), guest.id.clone())
                {
                    error!("Error disabling resource for guest {:?}", err);
                };
            }
            Err(LeaveFailedState::PersistedStateGoneMissingGoneWild) => {
                error!("Guest state could not be loaded...? {}", module.name());
            }
            Err(LeaveFailedState::NotInModule) => {
                error!(
                    "Guest is not in module {} but tried to leave it, this should not happen.",
                    module.name()
                );
            }
        }
    }

    pub fn try_enter_module(
        guest: &mut Guest,
        module_enter_slot: &ModuleEnterSlot,
        module: &mut DynamicGameModule,
        resource_module: &mut ResourceModule,
    ) {
        match module.try_enter(guest, module_enter_slot) {
            Ok(EnterSuccessState::Entered) => {
                guest.current_module = Some(module.name());
                guest.pending_module_exit = None;
                if let Err(err) =
                    resource_module.activate_resources_for_guest(module.name(), guest.id)
                {
                    error!(
                        "Error activating resource for guest: {}",
                        fix_intellij_error_bug(&err)
                    );
                };
            }
            Err(EnterFailedState::PersistedStateGoneMissingGoneWild) => {
                error!("Guest state could not be loaded...? {}", module.name());
            }
            Err(EnterFailedState::GameInstanceNotFoundWTF) => {
                error!("Game instance not found wtf? {}", module.name());
            }
            Err(EnterFailedState::AlreadyEntered) => {
                error!(
                    "Guest already entered {}, this should not happen.",
                    module.name()
                );
            }
        }
    }

    pub fn process_system_event(
        &mut self,
        system_event: ModuleToSystem,
    ) -> Result<(), ProcessModuleEventError> {
        let GuestEvent {
            guest_id,
            event_type,
        } = system_event;

        let guest = safe_unwrap(
            self.guests.get_mut(&guest_id),
            ProcessModuleEventError::GuestNotFound,
        )?;

        match event_type {
            ModuleToSystemEvent::GuestStateChange(state_change) => {
                if let Some(communication_event) = ConductorModule::process_guest_state_change(
                    guest,
                    state_change,
                    &mut self.persistence_module,
                )? {
                    if let CommunicationEvent::ShowGlobalMessage(_message) = &communication_event {
                        let guest_ids: Vec<GuestId> = self.guests.keys().cloned().collect();
                        for guest_id in guest_ids {
                            Self::send_communication_event_to_guest(
                                &mut self.guests,
                                &mut self.websocket_module,
                                guest_id,
                                &communication_event,
                            )?;
                        }
                    } else {
                        Self::send_communication_event_to_guest(
                            &mut self.guests,
                            &mut self.websocket_module,
                            guest_id,
                            &communication_event,
                        )?;
                    }
                }
            }
            ModuleToSystemEvent::GlobalMessage(message) => {
                let guest_ids: Vec<GuestId> = self.guests.keys().cloned().collect();
                for guest_id in guest_ids {
                    Self::send_communication_event_to_guest(
                        &mut self.guests,
                        &mut self.websocket_module,
                        guest_id,
                        &CommunicationEvent::ShowGlobalMessage(message.clone()),
                    )?;
                }
            }
            ModuleToSystemEvent::ToastMessage(toast_alert_level, message) => {
                Self::send_communication_event_to_guest(
                    &mut self.guests,
                    &mut self.websocket_module,
                    guest_id,
                    &CommunicationEvent::Toast(toast_alert_level, message),
                )?;
            }
        }

        Ok(())
    }

    pub fn send_communication_event_to_guest(
        guests: &mut HashMap<Snowflake, Guest>,
        websocket_module: &mut WebsocketModule,
        guest_id: Snowflake,
        event: &CommunicationEvent,
    ) -> Result<(), ProcessModuleEventError> {
        if let Ok(message_as_string) = serde_json::to_string(event) {
            if let Some(guest) = guests.get(&guest_id) {
                Self::send_to_guest(guest, websocket_module, message_as_string);
            }
            Ok(())
        } else {
            Err(ProcessModuleEventError::CouldNotSerializeCommunicationEvent)
        }
    }

    pub fn process_guest_state_change(
        guest: &mut Guest,
        guest_state_change: GuestStateChange,
        persistence_module: &mut PersistenceModule,
    ) -> Result<Option<CommunicationEvent>, ProcessModuleEventError> {
        match guest_state_change {
            GuestStateChange::ExitModule(module_exit_slot) => {
                guest.pending_module_exit = Some(module_exit_slot);

                Ok(None)
            }
            GuestStateChange::FoundSecret(name, _module_name) => {
                if let Some(persisted_guest_state) = &mut guest.persisted_guest {
                    let secret =
                        persistence_module.add_secret_found(name, persisted_guest_state.info.id)?;
                    persisted_guest_state.secrets_found.push(secret);

                    return Ok(Some(CommunicationEvent::ShowGlobalMessage(format!(
                        "{} found a shard!",
                        persisted_guest_state.info.display_name
                    ))));
                }

                Ok(None)
            }
        }
    }

    fn handle_times_joined(
        persistence_module: &mut PersistenceModule,
        mut persisted_guest: &mut PersistedGuest,
    ) -> Result<usize, PersistenceError> {
        let now = Utc::now().naive_utc();

        let last_joined_or_never = persisted_guest
            .info
            .last_time_joined
            .unwrap_or_else(|| NaiveDateTime::new(NaiveDate::default(), NaiveTime::default()));

        if (now - last_joined_or_never).num_hours() > LOGGED_IN_TODAY_DELAY_IN_HOURS {
            debug!(
                "Last time join was longer than {} hours ago.",
                LOGGED_IN_TODAY_DELAY_IN_HOURS
            );
            persisted_guest.info.times_joined += 1;
        }

        persisted_guest.info.last_time_joined = Some(now);

        persistence_module.update_persisted_guest_state(UpdatePersistedGuestState {
            id: persisted_guest.info.id,
            last_time_joined: persisted_guest.info.last_time_joined,
            times_joined: Some(persisted_guest.info.times_joined),
            is_tester: None,
            is_observer: None,
        })
    }

    pub fn process_position_event(
        &mut self,
        game_position: GamePosition,
    ) -> Result<(), ProcessGameEventError> {
        let GuestEvent {
            guest_id,
            event_type,
        } = game_position;
        if let Ok(message_as_string) =
            serde_json::to_string(&CommunicationEvent::PositionEvent(event_type))
        {
            if let Some(guest) = self.guests.get(&guest_id) {
                Self::send_to_guest(guest, &mut self.websocket_module, message_as_string);
            }
        } else {
            return Err(ProcessGameEventError::CouldNotSerializePosition);
        }

        Ok(())
    }

    pub fn process_game_event(
        &mut self,
        game_event: GameSystemToGuest,
    ) -> Result<(), ProcessGameEventError> {
        let GuestEvent {
            guest_id,
            event_type,
        } = game_event;
        if let Ok(message_as_string) =
            serde_json::to_string(&CommunicationEvent::GameSystemEvent(event_type))
        {
            if let Some(guest) = self.guests.get(&guest_id) {
                Self::send_to_guest(guest, &mut self.websocket_module, message_as_string);
            }
        } else {
            error!("Error serializing resource event!");
        }

        Ok(())
    }

    pub fn process_events_from_modules(&mut self) {
        let mut module_to_system_events = Vec::new();
        let mut game_system_to_guest_events = Vec::new();
        let mut position_events = Vec::new();
        for module_communication in self.module_communication_map.values() {
            module_to_system_events.extend(
                module_communication
                    .receiver
                    .module_to_system_receiver
                    .drain(),
            );
            game_system_to_guest_events.extend(
                module_communication
                    .receiver
                    .game_system_to_guest_receiver
                    .drain(),
            );
            position_events.extend(module_communication.receiver.position_receiver.drain());
        }

        for event in module_to_system_events {
            match self.process_system_event(event) {
                Ok(()) => (),
                Err(ProcessModuleEventError::PersistenceError(err)) => {
                    error!(
                        "Something went wrong while trying to persist guest state! {:?}",
                        err
                    );
                }
                Err(ProcessModuleEventError::GuestNotFound) => {
                    error!("Could not find guest, this should never happen!");
                }
                Err(ProcessModuleEventError::CouldNotSerializeCommunicationEvent) => {
                    error!("Could not serialize communication event for system event.");
                }
            }
        }

        for event in game_system_to_guest_events {
            match self.process_game_event(event) {
                Ok(()) => (),
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }

        for event in position_events {
            match self.process_position_event(event) {
                Ok(()) => (),
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }
    }

    pub fn send_to_current_guest_module(
        &mut self,
        guest_id: Snowflake,
        event: GuestEvent<SystemToModuleEvent>,
    ) {
        if let Some(guest) = self.guests.get(&guest_id) {
            if let Some(current_module) = &guest.current_module {
                if let Some(communication) = self.module_communication_map.get_mut(current_module) {
                    if let Err(err) = communication.sender.system_to_module_sender.send(event) {
                        error!("Could not send event to module! {}", err);
                    }
                }
            }
        }
    }

    pub fn process_events_from_guest(&mut self) {
        for (guest_id, guest) in &self.guests {
            if let Some(ws_connection_id) = &guest.ws_connection_id {
                for message in self.websocket_module.drain_events(ws_connection_id) {
                    match serde_json::from_str::<GuestTo>(message.to_string().as_str()) {
                        Ok(guest_to) => match guest_to {
                            GuestTo::GuestToSystemEvent(event) => {
                                ConductorModule::process_guest_to_system_event(
                                    event,
                                    *guest_id,
                                    &mut self.login_manager,
                                );
                            }
                            GuestTo::GuestToModuleEvent(event) => {
                                if let Some(module_name_guest_is_currently_in) =
                                    &guest.current_module
                                {
                                    if let Some(module_communication) = self
                                        .module_communication_map
                                        .get(module_name_guest_is_currently_in)
                                    {
                                        if let Err(err) = ConductorModule::send_event_to_module(
                                            module_communication,
                                            *guest_id,
                                            event,
                                        ) {
                                            error!("process_events_from_guest, send_event_to_module {:?}", err);
                                        }
                                    }
                                }
                            }
                        },
                        Err(err) => {
                            error!("process_events_from_guest - Error trying to parse guest_to event {:?}", err);
                        }
                    }
                }
            }
        }
    }

    fn process_guest_to_system_event(
        event: GuestToSystemEvent,
        guest_id: GuestId,
        login_manager: &mut LoginManager,
    ) {
        match event {
            GuestToSystemEvent::ProviderLoggedIn(provider_logged_in) => {
                login_manager.add_provider_login(guest_id, provider_logged_in);
            }
            GuestToSystemEvent::Ping => {}
        }
    }

    fn send_event_to_module(
        module_communication: &ModuleIO,
        guest_id: Snowflake,
        event: GuestToModuleEvent,
    ) -> Result<(), SendEventToModuleError> {
        module_communication
            .sender
            .guest_to_module_sender
            .send(GuestToModule {
                guest_id,
                event_type: event,
            })?;

        Ok(())
    }
    fn process_logins(&mut self) {
        let guests = &mut self.guests;
        let system_communication_sender = &mut self.system_to_guest_communication.sender;
        let provider_id_to_guest_map = &mut self.provider_id_to_guest_map;
        let persistence_module = &mut self.persistence_module;
        self.login_manager.process_running_logins(|res| match res {
            Ok((guest_id, login_data)) => {
                if let Some(guest) = guests.get_mut(&guest_id) {
                    match Self::handle_user_login(
                        provider_id_to_guest_map,
                        persistence_module,
                        login_data,
                        guest,
                    ) {
                        Ok(()) => {
                            send_and_log_error(
                                system_communication_sender,
                                (
                                    guest.id,
                                    CommunicationEvent::Signal(SignalToGuest::LoginSuccess),
                                ),
                            );
                        }
                        Err(HandleLoginError::AlreadyLoggedIn) => {
                            debug!("User is already logged in!")
                        }
                        Err(HandleLoginError::PersistenceError(err)) => {
                            error!("Could not login: {:?}", err);
                            send_and_log_error(
                                system_communication_sender,
                                (
                                    guest.id,
                                    CommunicationEvent::Signal(SignalToGuest::LoginFailed),
                                ),
                            );
                        }
                    }
                }
            }
            Err(error) => match error {
                LoginError::UserDidNotExistLongEnough(guest_id, time) => {
                    send_and_log_error(
                        system_communication_sender,
                        (
                            guest_id,
                            CommunicationEvent::Toast(
                                ToastAlertLevel::Error,
                                format!(
                                    "Your account is not older than {} days. Please ask shiku!",
                                    time
                                ),
                            ),
                        ),
                    );
                }
                LoginError::TwitchApiError(guest_id, twitch_api_error) => {
                    debug!(
                        "Could not login user due to twitch api error {:?}",
                        twitch_api_error
                    );
                    send_and_log_error(
                        system_communication_sender,
                        (
                            guest_id,
                            CommunicationEvent::Toast(
                                ToastAlertLevel::Error,
                                "Could not login because of login error. Please ask shiku!"
                                    .to_string(),
                            ),
                        ),
                    );
                }
            },
        })
    }

    fn handle_user_login(
        provider_id_to_guest_map: &mut HashMap<ProviderUserId, Snowflake>,
        persistence_module: &mut PersistenceModule,
        login_data: LoginData,
        guest: &mut Guest,
    ) -> Result<(), HandleLoginError> {
        let mut persisted_guest = persistence_module.lazy_get_persisted_guest_by_provider_id(
            &login_data.provider_user_id,
            &login_data.display_name,
        )?;

        Self::handle_times_joined(persistence_module, &mut persisted_guest)?;

        if provider_id_to_guest_map
            .get(&login_data.provider_user_id)
            .is_some()
        {
            return Err(HandleLoginError::AlreadyLoggedIn);
        }

        provider_id_to_guest_map.insert(login_data.provider_user_id.clone(), guest.id);

        guest.login_data = Some(login_data);
        guest.pending_module_exit = Some("".into());
        guest.persisted_guest = Some(persisted_guest);

        Ok(())
    }
    fn send_system_events_to_guests(&mut self) {
        for (guest_id, communication_event) in self.system_to_guest_communication.receiver.drain() {
            if let Err(err) = Self::send_communication_event_to_guest(
                &mut self.guests,
                &mut self.websocket_module,
                guest_id,
                &communication_event,
            ) {
                error!("{:?}", err);
            }
        }
    }
}
