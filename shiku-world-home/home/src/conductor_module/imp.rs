use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::time::Instant;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use flume::{unbounded, Sender};
use log::{debug, error, warn};
use rhai::Module;
use snowflake::SnowflakeIdBucket;
use tungstenite::protocol::frame::coding::CloseCode;

use crate::conductor_module::admin_to_system_events::handle_admin_to_system_event;
use crate::conductor_module::def::{ConductorModule, ResourceToModuleMap};
use crate::conductor_module::errors::{
    HandleLoginError, ProcessGameEventError, ProcessModuleEventError, SendEventToModuleError,
};
use crate::conductor_module::game_instances::create_game_instance_manager;
use crate::core::blueprint::def::{
    BlueprintResource, BlueprintService, GidMap, ModuleId, ResourceKind, ResourcePath,
    TerrainParams, Tileset,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::guest::{
    ActorId, Actors, Admin, Guest, LoginData, ModuleEnterSlot, ProviderUserId,
};
use crate::core::module::{
    AdminToSystemEvent, CommunicationEvent, EditorEvent, EnterFailedState, EnterSuccessState,
    GamePosition, GameSystemToGuest, GuestEvent, GuestStateChange, GuestTo, GuestToModule,
    GuestToModuleEvent, GuestToSystemEvent, LeaveFailedState, LeaveSuccessState, ModuleIO,
    ModuleInstanceEvent, ModuleName, ModuleState, ModuleToSystem, ModuleToSystemEvent,
    SignalToMedium, SystemCommunicationIO, SystemToModule, SystemToModuleEvent, ToastAlertLevel,
};
use crate::core::module_system::game_instance::{GameInstanceId, GameInstanceManager};
use crate::core::module_system::world::WorldId;
use crate::core::{blueprint, send_and_log_error, send_and_log_error_custom};
use crate::core::{safe_unwrap, Snowflake, LOGGED_IN_TODAY_DELAY_IN_HOURS};
use crate::login::login_manager::{LoginError, LoginManager};
use crate::persistence_module::models::{PersistedGuest, UpdatePersistedGuestState};
use crate::persistence_module::{PersistenceError, PersistenceModule};
use crate::resource_module::def::ResourceBundle;
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
        self.send_system_events_to_admins();
        self.process_logins();
        self.send_load_events();
        self.process_picture_update_events();

        self.handle_timeouts();

        self.handle_admin_events().await;
    }

    pub fn update_resource_to_module_map(
        resource_to_module_map: &mut ResourceToModuleMap,
        module_id: &ModuleId,
        current_resources: &Vec<BlueprintResource>,
        updated_resources: &Vec<BlueprintResource>,
    ) {
        let current_script_set: HashSet<String> =
            current_resources.iter().map(|r| r.path.clone()).collect();
        let updated_script_set: HashSet<String> =
            updated_resources.iter().map(|r| r.path.clone()).collect();
        for insertion in updated_script_set.difference(&current_script_set) {
            resource_to_module_map
                .entry(insertion.clone())
                .or_default()
                .insert(module_id.clone());
        }
        for deletion in current_script_set.difference(&updated_script_set) {
            resource_to_module_map
                .entry(deletion.clone())
                .or_default()
                .remove(module_id);
        }
    }

    fn check_admin_login(login_data: &LoginData) -> bool {
        login_data.provider_user_id == "52657886"
    }

    async fn handle_admin_events(&mut self) {
        for admin in &mut self.admins.values() {
            if let Some(ws_connection_id) = admin.ws_connection_id {
                for message in self.websocket_module.drain_events(&ws_connection_id) {
                    match serde_json::from_str::<AdminToSystemEvent>(message.to_string().as_str()) {
                        Ok(event) => {
                            if let AdminToSystemEvent::Ping = event {
                                return;
                            }
                            if !admin.is_logged_in {
                                if let AdminToSystemEvent::ProviderLoggedIn(provider) = event {
                                    debug!("PROVIDER LOGGED IN FOR ADMIN!");
                                    self.login_manager.add_provider_login(admin.id, provider);
                                } else {
                                    error!(
                    "Admin tried to do something other than logging in while not being logged in!"
                );
                                }
                                continue;
                            }
                            handle_admin_to_system_event(
                                &mut self.module_communication_map,
                                &mut self.web_server_module,
                                &mut self.resource_module,
                                &mut self.module_map,
                                &mut self.resource_to_module_map,
                                &mut self.system_to_admin_communication.sender,
                                admin,
                                event,
                            )
                            .await;
                        }
                        Err(err) => error!("Failed to parse admin event! {:?}", err),
                    }
                }
            }
        }
    }

    fn handle_timeouts(&mut self) {
        for (guest_id, connection_lost_time) in &self.guest_timeout_map {
            if connection_lost_time.elapsed().as_secs() > 30 {
                self.timeouts.push(*guest_id);
            }
        }
        for guest_id in self.timeouts.drain(..) {
            if let Some(guest) = self.guests.remove(&guest_id) {
                if let Some(module_id) = &guest.current_module_id {
                    if let Some(module) = self.module_map.get_mut(module_id) {
                        if let Err(err) = module.try_leave(&guest) {
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
            debug!("Guest removed {}.", guest_id);
        }
    }

    pub fn update_modules(&mut self) {
        for instance_manager in self.module_map.values_mut() {
            instance_manager.update();
        }
    }

    pub async fn new(
        websocket_module: WebsocketModule,
        blueprint_service: BlueprintService,
        blueprint: blueprint::def::Conductor,
    ) -> ConductorModule {
        let snowflake_gen = SnowflakeIdBucket::new(1, 1);
        let mut module_communication_map = HashMap::new();
        let mut module_map = HashMap::new();
        let mut resource_module = ResourceModule::new().await;
        let (sender, receiver) = unbounded();
        let system_to_guest_communication = SystemCommunicationIO { receiver, sender };
        let (sender, receiver) = unbounded();
        let system_to_admin_communication = SystemCommunicationIO { receiver, sender };

        let modules = Blueprint::get_all_modules().unwrap();
        let mut resource_to_module_map = HashMap::new();
        for module in modules {
            create_game_instance_manager(
                module.clone(),
                &mut module_map,
                &mut resource_module,
                &mut resource_to_module_map,
                &mut module_communication_map,
            )
            .unwrap();
        }

        ConductorModule {
            blueprint,
            blueprint_service,
            websocket_module,
            resource_module,
            persistence_module: PersistenceModule::new(),
            web_server_module: WebServerModule::new(),
            login_manager: LoginManager::new(),
            snowflake_gen,
            module_connection_map: HashMap::new(),
            resource_to_module_map: HashMap::new(),
            guests: HashMap::new(),
            admins: HashMap::new(),

            ws_to_guest_map: HashMap::new(),
            ws_to_admin_map: HashMap::new(),
            provider_id_to_guest_map: HashMap::new(),
            provider_id_to_admin_map: HashMap::new(),

            session_id_to_guest_map: HashMap::new(),
            session_id_to_admin_map: HashMap::new(),
            guest_timeout_map: HashMap::new(),

            timeouts: Vec::new(),
            module_map,

            module_communication_map,
            system_to_guest_communication,
            system_to_admin_communication,
        }
    }

    pub fn handle_new_ws_connections(&mut self) {
        for (connection_id, ticket) in self.websocket_module.handle_new_ws_connections() {
            debug!("{:?}", ticket);
            if let Some(true) = ticket.admin_login {
                debug!("Admin ready to start their session!");

                let admin_id_from_session_id = self
                    .session_id_to_admin_map
                    .get(&ticket.session_id.unwrap_or_default())
                    .unwrap_or(&0);

                debug!("{}, {:?}", admin_id_from_session_id, self.admins);

                let admin_id: Snowflake =
                    if let Some(admin) = self.admins.get_mut(admin_id_from_session_id) {
                        debug!("Admin already existed with their session!");
                        if let Some(existing_connection_id) = admin.ws_connection_id {
                            debug!("Admin already connected o_o1");
                            self.websocket_module.close_connection(
                                &existing_connection_id,
                                CloseCode::Normal,
                                "Connected elsewhere".into(),
                            );
                        }

                        self.ws_to_admin_map.insert(connection_id, admin.id);
                        admin.ws_connection_id = Some(connection_id);
                        admin.id
                    } else {
                        debug!("Created new admin!");
                        let admin_id = self.snowflake_gen.get_id();
                        let session_id = self.snowflake_gen.get_id().to_string();
                        self.admins.insert(
                            admin_id,
                            Admin {
                                session_id: session_id.clone(),
                                id: admin_id,
                                login_data: None,
                                is_logged_in: false,
                                ws_connection_id: Some(connection_id),
                            },
                        );
                        self.ws_to_admin_map.insert(connection_id, admin_id);
                        self.session_id_to_admin_map.insert(session_id, admin_id);
                        admin_id
                    };

                if let Some(admin) = self.admins.get(&admin_id) {
                    if let Ok(event_as_string) =
                        serde_json::to_string(&CommunicationEvent::ConnectionReady((
                            admin.session_id.clone(),
                            admin.login_data.is_none(),
                        )))
                    {
                        Self::send_to_admin(admin, &mut self.websocket_module, event_as_string);
                    } else {
                        error!("Could not parse ConnectionReady enum, wtf?");
                    }
                }
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

                    if let (Some(current_module_id), Some(current_instance_id)) =
                        (&guest.current_module_id, &guest.current_instance_id)
                    {
                        if let Some(module_communication) =
                            self.module_communication_map.get_mut(current_module_id)
                        {
                            send_and_log_error_custom(
                                &mut module_communication.sender.system_to_module_sender,
                                ModuleInstanceEvent {
                                    module_id: current_module_id.clone(),
                                    instance_id: current_instance_id.clone(),
                                    world_id: None,
                                    event_type: SystemToModuleEvent::Reconnected(guest.id),
                                },
                                "Error sending reconnect event",
                            );
                            if let Some(module) = self.module_map.get(current_module_id) {
                                if let Some(terrain_params) = module
                                    .get_terrain_params_for_guest(&guest.id, current_instance_id)
                                {
                                    match BlueprintService::load_module_tilesets(
                                        &module.module_blueprint.resources,
                                    ) {
                                        Ok(tilesets) => Self::send_prepare_game_event(
                                            &guest,
                                            &mut self.resource_module,
                                            &mut self.websocket_module,
                                            current_module_id,
                                            current_instance_id,
                                            terrain_params,
                                            tilesets,
                                            module.module_blueprint.gid_map.clone(),
                                        ),
                                        Err(err) => {
                                            error!("Could not load tilesets for module! {:?}", err)
                                        }
                                    }
                                }
                            }
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
                current_module_id: None,
                current_instance_id: None,
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
            if let Some(admin_id) = self.ws_to_admin_map.remove(&connection_id) {
                debug!("admin connection lost {:?}", &admin_id);
                if let Some(admin) = self.admins.get_mut(&admin_id) {
                    admin.ws_connection_id = None;
                } else {
                    debug!("Admin {:?} logged in somewhere else it seems.", admin_id);
                }
                continue;
            }

            if let Some(guest_id) = self.ws_to_guest_map.remove(&connection_id) {
                if let Some(guest) = self.guests.get_mut(&guest_id) {
                    debug!("guest connection lost {:?}", &guest_id);
                    guest.ws_connection_id = None;

                    self.guest_timeout_map.insert(guest_id, Instant::now());

                    if let (Some(current_module_id), Some(current_instance_id)) =
                        (&guest.current_module_id, &guest.current_instance_id)
                    {
                        if let Some(module_communication) =
                            self.module_communication_map.get_mut(current_module_id)
                        {
                            if let Err(err) = module_communication
                                .sender
                                .system_to_module_sender
                                .send(ModuleInstanceEvent {
                                    module_id: current_module_id.clone(),
                                    instance_id: current_instance_id.clone(),
                                    world_id: None,
                                    event_type: SystemToModuleEvent::Disconnected(guest_id),
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

    pub fn process_picture_update_events(&mut self) {
        self.resource_module.receive_all_picture_updates();
        self.resource_module.process_picture_updates();
    }

    pub fn send_load_events(&mut self) {
        for (actor_id, module_id, event_type) in self.resource_module.drain_load_events() {
            if let Ok(message_as_string) =
                serde_json::to_string(&CommunicationEvent::ResourceEvent(module_id, event_type))
            {
                debug!("Sending load event to actor");
                if let Some(guest) = self.guests.get(&actor_id) {
                    Self::send_to_guest(guest, &mut self.websocket_module, message_as_string);
                } else if let Some(admin) = self.admins.get(&actor_id) {
                    Self::send_to_admin(admin, &mut self.websocket_module, message_as_string);
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
            debug!("Could not send to guest '{:?}' no active connection", guest);
        }
    }

    pub fn send_to_admin(
        admin: &Admin,
        websocket_module: &mut WebsocketModule,
        event_as_string: String,
    ) {
        if let Some(ws_connection_id) = &admin.ws_connection_id {
            websocket_module.send_event(ws_connection_id, event_as_string);
        } else {
            debug!("Could not send to guest '{:?}' no active connection", admin);
        }
    }

    pub fn move_guests(&mut self) {
        for guest in self.guests.values_mut() {
            if let Some(module_exit_slot) = &guest.pending_module_exit {
                if let Some((target_module_name, module_enter_slot)) =
                    self.module_connection_map.get(module_exit_slot)
                {
                    if let Some(current_module_id) = &guest.current_module_id {
                        if current_module_id == target_module_name {
                            error!("current module {} and target module {} are the same, this should never happen!", current_module_id, target_module_name);
                            continue;
                        }
                        let current_module_option = self.module_map.get_mut(current_module_id);
                        if let Some(current_module) = current_module_option {
                            ConductorModule::try_leave_module(
                                guest,
                                current_module,
                                &mut self.resource_module,
                            );
                        } else {
                            error!(
                                "Module {} did not exist, so user cannot leave it, guest is stuck!",
                                current_module_id
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
                            &mut self.websocket_module,
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
        module: &mut GameInstanceManager,
        resource_module: &mut ResourceModule,
    ) {
        match module.try_leave(guest) {
            Ok((_instance_id, LeaveSuccessState::Left)) => {
                guest.current_module_id = None;
                if let Err(err) = resource_module
                    .disable_module_resource_updates(module.module_blueprint.id.clone(), &guest.id)
                {
                    error!("Error disabling resource for guest {:?}", err);
                };
            }
            Err(LeaveFailedState::PersistedStateGoneMissingGoneWild) => {
                error!(
                    "Guest state could not be loaded...? {}",
                    module.module_blueprint.name
                );
            }
            Err(LeaveFailedState::NotInModule) => {
                error!(
                    "Guest is not in module {} but tried to leave it, this should not happen.",
                    module.module_blueprint.name
                );
            }
        }
    }

    pub fn try_enter_module(
        guest: &mut Guest,
        module_enter_slot: &ModuleEnterSlot,
        module: &mut GameInstanceManager,
        resource_module: &mut ResourceModule,
        websocket_module: &mut WebsocketModule,
    ) {
        let module_name = module.module_blueprint.name.clone();
        match module.try_enter(guest, module_enter_slot) {
            Ok((instance_id, EnterSuccessState::Entered)) => {
                debug!("Entered I think {:?}", instance_id);
                guest.current_module_id = Some(module_name.clone());
                guest.current_instance_id = Some(instance_id.clone());
                guest.pending_module_exit = None;
                resource_module.activate_module_resource_updates(module_name.clone(), &guest.id);
                if let Some(terrain_params) =
                    module.get_terrain_params_for_guest(&guest.id, &instance_id)
                {
                    match BlueprintService::load_module_tilesets(&module.module_blueprint.resources)
                    {
                        Ok(tilesets) => Self::send_prepare_game_event(
                            &guest,
                            resource_module,
                            websocket_module,
                            &module_name,
                            &instance_id,
                            terrain_params,
                            tilesets,
                            module.module_blueprint.gid_map.clone(),
                        ),
                        Err(err) => {
                            error!("Could not load tilesets for module! {:?}", err)
                        }
                    }
                }
            }
            Err(EnterFailedState::PersistedStateGoneMissingGoneWild) => {
                error!("Guest state could not be loaded...? {}", module_name);
            }
            Err(EnterFailedState::GameInstanceNotFoundWTF) => {
                error!("Game instance not found wtf? {}", module_name);
            }
            Err(EnterFailedState::AlreadyEntered) => {
                error!(
                    "Guest already entered {}, this should not happen.",
                    module_name
                );
            }
        }
    }

    fn send_prepare_game_event(
        guest: &&mut Guest,
        resource_module: &mut ResourceModule,
        websocket_module: &mut WebsocketModule,
        module_id: &ModuleId,
        instance_id: &GameInstanceId,
        terrain_params: TerrainParams,
        tilesets: Vec<Tileset>,
        gid_map: GidMap,
    ) {
        if let Ok(resources) = resource_module.get_active_resources_for_module(module_id, &guest.id)
        {
            if let Err(err) = Self::send_communication_event_to_guest_direct(
                guest,
                websocket_module,
                &CommunicationEvent::PrepareGame(
                    module_id.clone(),
                    instance_id.clone(),
                    None,
                    ResourceBundle {
                        name: "Init".into(),
                        assets: resources,
                    },
                    terrain_params,
                    tilesets,
                    gid_map,
                ),
            ) {
                error!("Cold not send communicastion event to guest {:?}", err);
            }
        }
    }

    pub fn process_system_event(
        &mut self,
        system_event: ModuleToSystem,
    ) -> Result<(), ProcessModuleEventError> {
        match system_event {
            ModuleToSystemEvent::GameInstanceClosed(module_id, game_instance_id) => {
                for admin in self.admins.values() {
                    send_and_log_error(
                        &mut self.system_to_admin_communication.sender,
                        (
                            admin.id,
                            CommunicationEvent::EditorEvent(EditorEvent::ModuleInstanceClosed(
                                module_id.clone(),
                                game_instance_id.clone(),
                            )),
                        ),
                    )
                }
            }
            ModuleToSystemEvent::GameInstanceCreated(module_id, game_instance_id) => {
                for admin in self.admins.values() {
                    send_and_log_error(
                        &mut self.system_to_admin_communication.sender,
                        (
                            admin.id,
                            CommunicationEvent::EditorEvent(EditorEvent::ModuleInstanceOpened(
                                module_id.clone(),
                                game_instance_id.clone(),
                            )),
                        ),
                    )
                }
            }
            ModuleToSystemEvent::GuestStateChange(guest_id, state_change) => {
                let guest = safe_unwrap(
                    self.guests.get_mut(&guest_id),
                    ProcessModuleEventError::GuestNotFound,
                )?;
                if let Some(communication_event) = ConductorModule::process_guest_state_change(
                    guest,
                    state_change,
                    &mut self.persistence_module,
                )? {
                    if let CommunicationEvent::ShowGlobalMessage(_message) = &communication_event {
                        let guest_ids: Vec<ActorId> = self.guests.keys().cloned().collect();
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
                let guest_ids: Vec<ActorId> = self.guests.keys().cloned().collect();
                for guest_id in guest_ids {
                    Self::send_communication_event_to_guest(
                        &mut self.guests,
                        &mut self.websocket_module,
                        guest_id,
                        &CommunicationEvent::ShowGlobalMessage(message.clone()),
                    )?;
                }
            }
            ModuleToSystemEvent::ToastMessage(guest_id, toast_alert_level, message) => {
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
        if let Some(guest) = guests.get(&guest_id) {
            debug!("Sending to guest {:?}", guest);
            return Self::send_communication_event_to_guest_direct(guest, websocket_module, event);
        }

        Err(ProcessModuleEventError::GuestNotFound)
    }

    pub fn send_communication_event_to_guest_direct(
        guest: &Guest,
        websocket_module: &mut WebsocketModule,
        event: &CommunicationEvent,
    ) -> Result<(), ProcessModuleEventError> {
        if let Ok(message_as_string) = serde_json::to_string(event) {
            Self::send_to_guest(guest, websocket_module, message_as_string);
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
        debug!("{:?} {:?}", guest_id, event_type);
        /*if let Ok(message_as_string) = serde_json::to_string(&CommunicationEvent::PositionEvent(
            event_type.0,
            event_type.1,
            event_type.2,
            event_type.3,
        )) {
            if let Some(guest) = self.guests.get(&guest_id) {
                Self::send_to_guest(guest, &mut self.websocket_module, message_as_string);
            }
        } else {
            return Err(ProcessGameEventError::CouldNotSerializePosition);
        }*/

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
        let ModuleInstanceEvent {
            instance_id,
            module_id,
            world_id,
            event_type,
        } = event_type;
        if let Ok(message_as_string) = serde_json::to_string(&CommunicationEvent::GameSystemEvent(
            module_id,
            instance_id,
            world_id,
            event_type,
        )) {
            if let Some(guest) = self.guests.get(&guest_id) {
                Self::send_to_guest(guest, &mut self.websocket_module, message_as_string);
            } else if let Some(admin) = self.admins.get(&guest_id) {
                Self::send_to_admin(admin, &mut self.websocket_module, message_as_string);
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

    pub fn send_to_current_guest_module(&mut self, guest_id: Snowflake, event: SystemToModule) {
        if let Some(guest) = self.guests.get(&guest_id) {
            if let Some(current_module_id) = &guest.current_module_id {
                if let Some(communication) =
                    self.module_communication_map.get_mut(current_module_id)
                {
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
                    println!("{}", message.to_string().as_str());
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
                                if let (Some(current_module_id), Some(current_instance_id)) =
                                    (&guest.current_module_id, &guest.current_instance_id)
                                {
                                    if let Some(module_communication) =
                                        self.module_communication_map.get(current_module_id)
                                    {
                                        if let Err(err) = ConductorModule::send_event_to_module(
                                            module_communication,
                                            *guest_id,
                                            current_module_id.clone(),
                                            current_instance_id.clone(),
                                            None,
                                            event,
                                        ) {
                                            error!("process_events_from_guest, send_event_to_module {:?}", err);
                                        }
                                    }
                                } else {
                                    error!("Could not send event to module! module {:?}, instance: {:?}", guest.current_module_id, guest.current_instance_id);
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
        guest_id: ActorId,
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
        guest_id: ActorId,
        module_id: ModuleId,
        instance_id: GameInstanceId,
        world_id: Option<WorldId>,
        event: GuestToModuleEvent,
    ) -> Result<(), SendEventToModuleError> {
        module_communication
            .sender
            .guest_to_module_sender
            .send(GuestToModule {
                guest_id,
                event_type: ModuleInstanceEvent {
                    module_id,
                    instance_id,
                    world_id,
                    event_type: event,
                },
            })?;

        Ok(())
    }

    fn process_logins(&mut self) {
        let guests = &mut self.guests;
        let admins = &mut self.admins;
        let mut system_to_guest_communication_sender =
            &mut self.system_to_guest_communication.sender;
        let mut system_to_admin_communication_sender =
            &mut self.system_to_admin_communication.sender;
        let provider_id_to_guest_map = &mut self.provider_id_to_guest_map;
        let provider_id_to_admin_map = &mut self.provider_id_to_admin_map;
        let persistence_module = &mut self.persistence_module;
        let websocket_module = &mut self.websocket_module;
        let ws_to_guest_map = &mut self.ws_to_guest_map;
        let ws_to_admin_map = &mut self.ws_to_admin_map;
        let session_id_to_guest_map = &mut self.session_id_to_guest_map;
        let session_id_to_admin_map = &mut self.session_id_to_admin_map;
        self.login_manager.process_running_logins(|res| match res {
            Ok((actor_id, login_data)) => {
                debug!("login {} {:?}", actor_id, login_data);
                if guests.contains_key(&actor_id) {
                    Self::handle_actor_login_result(system_to_guest_communication_sender, Self::handle_actor_login(
                        provider_id_to_guest_map,
                        websocket_module,
                        &login_data,
                        &actor_id,
                        guests,
                    ws_to_guest_map,
                        session_id_to_guest_map,
                        |login_data, guest| {
                            if let Err(err) = Self::handle_guest_persistence(persistence_module, login_data, guest) {
                                error!("Oh oh! There was an error while trying to get guest persistence!!! {:?}", err)
                            }
                        }));
                } else {
                    Self::handle_actor_login_result(system_to_admin_communication_sender, Self::handle_actor_login(
                        provider_id_to_admin_map,
                        websocket_module,
                        &login_data,
                        &actor_id,
                        admins,
                        ws_to_admin_map,
                        session_id_to_admin_map,
                        |_, _| {}));
                };
            }
            Err(error) => match error {
                LoginError::UserDidNotExistLongEnough(actor_id, time) => {
                    let sender = if guests.contains_key(&actor_id) {&mut system_to_guest_communication_sender} else {&mut system_to_admin_communication_sender};
                    send_and_log_error(
                        *sender,
                        (
                            actor_id,
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
                LoginError::TwitchApiError(actor_id, twitch_api_error) => {
                    debug!(
                        "Could not login user due to twitch api error {:?}",
                        twitch_api_error
                    );
                    let sender = if guests.contains_key(&actor_id) {&mut system_to_guest_communication_sender} else {&mut system_to_admin_communication_sender};
                    send_and_log_error(
                        sender,
                        (
                            actor_id,
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

    fn handle_actor_login_result(
        sender: &mut Sender<(ActorId, CommunicationEvent)>,
        result: Result<ActorId, HandleLoginError>,
    ) {
        match result {
            Ok(actor_id) => {
                debug!("handle login was successful for {}", actor_id);
                send_and_log_error(
                    sender,
                    (
                        actor_id,
                        CommunicationEvent::Signal(SignalToMedium::LoginSuccess),
                    ),
                );

                debug!("Sending was successful? for {}", actor_id);
            }
            Err(
                HandleLoginError::CouldNotFind(actor_id)
                | HandleLoginError::NotAuthorized(actor_id),
            ) => {
                error!("Actor could not login");
                send_and_log_error(
                    sender,
                    (
                        actor_id,
                        CommunicationEvent::Signal(SignalToMedium::LoginFailed),
                    ),
                );
            }
        }
    }

    fn handle_actor_login<T: Actors + Debug, F: FnMut(&LoginData, &mut T)>(
        provider_id_to_actor_map: &mut HashMap<ProviderUserId, Snowflake>,
        websocket_module: &mut WebsocketModule,
        login_data: &LoginData,
        actor_id: &Snowflake,
        actors: &mut HashMap<Snowflake, T>,
        ws_to_actor_map: &mut HashMap<Snowflake, Snowflake>,
        session_to_actor_map: &mut HashMap<String, Snowflake>,
        mut login_success_cb: F,
    ) -> Result<ActorId, HandleLoginError> {
        if !Self::check_admin_login(login_data) {
            return Err(HandleLoginError::NotAuthorized(*actor_id));
        }

        if let Some(already_logged_in_actor_id) =
            provider_id_to_actor_map.get(&login_data.provider_user_id)
        {
            if let Some(actor) = actors.remove(actor_id) {
                if let Some(already_logged_in_actor) = actors.get_mut(already_logged_in_actor_id) {
                    debug!(
                        "was already logged in {:?} \n removed was {:?}",
                        already_logged_in_actor, actor
                    );
                    debug!("before ws to actor map {:?}", ws_to_actor_map);
                    if let Some(ws_connection_id) = already_logged_in_actor.get_ws_connection_id() {
                        ws_to_actor_map.remove(&ws_connection_id);
                        session_to_actor_map.remove(already_logged_in_actor.get_session_id());
                        debug!("going to close {}", ws_connection_id);
                        websocket_module.close_connection(
                            &ws_connection_id,
                            CloseCode::Normal,
                            "Logged in elsewhere".into(),
                        );
                    } else {
                        debug!("Actor had no ws-connection, proceeding to take him over! Arrrrr");
                    }

                    if let Some(ws_connection_id) = actor.get_ws_connection_id() {
                        debug!("going to swap ws-connections");
                        already_logged_in_actor.set_ws_connection_id(ws_connection_id);
                        already_logged_in_actor.set_login_data(login_data.clone());
                        already_logged_in_actor.set_is_logged_in(true);
                        already_logged_in_actor.set_session_id(actor.get_session_id().clone());
                        session_to_actor_map.insert(
                            already_logged_in_actor.get_session_id().clone(),
                            *already_logged_in_actor_id,
                        );
                        ws_to_actor_map.insert(ws_connection_id, *already_logged_in_actor_id);
                        debug!(
                            "after swap already logged in {:?} \n current {:?}",
                            already_logged_in_actor, actor
                        );
                        debug!("after ws to actor map {:?}", ws_to_actor_map);
                        login_success_cb(login_data, already_logged_in_actor);
                        return Ok(*already_logged_in_actor_id);
                    }
                }
            }

            return Err(HandleLoginError::CouldNotFind(actor_id.clone()));
        }

        if let Some(actor) = actors.get_mut(actor_id) {
            provider_id_to_actor_map.insert(login_data.provider_user_id.clone(), actor.get_id());

            actor.set_login_data(login_data.clone());
            actor.set_is_logged_in(true);
            return Ok(*actor_id);
        }

        Err(HandleLoginError::CouldNotFind(actor_id.clone()))
    }

    fn handle_guest_persistence(
        persistence_module: &mut PersistenceModule,
        login_data: &LoginData,
        guest: &mut Guest,
    ) -> Result<(), PersistenceError> {
        let mut persisted_guest = persistence_module.lazy_get_persisted_guest_by_provider_id(
            &login_data.provider_user_id,
            &login_data.display_name,
        )?;

        Self::handle_times_joined(persistence_module, &mut persisted_guest)?;
        guest.persisted_guest = Some(persisted_guest);

        Ok(())
    }
    fn send_system_events_to_guests(&mut self) {
        if !self.system_to_guest_communication.receiver.is_empty() {
            debug!("There are messages!");
        }
        for (guest_id, communication_event) in self.system_to_guest_communication.receiver.drain() {
            debug!("Sending system event to guest? {:?}", communication_event);
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

    fn send_system_events_to_admins(&mut self) {
        for (admin_id, communication_event) in self.system_to_admin_communication.receiver.drain() {
            if let Some(admin) = self.admins.get(&admin_id) {
                if let Ok(message_as_string) = serde_json::to_string(&communication_event) {
                    Self::send_to_admin(admin, &mut self.websocket_module, message_as_string);
                } else {
                    error!(
                        "{:?}",
                        ProcessModuleEventError::CouldNotSerializeCommunicationEvent
                    );
                }
            }
        }
    }
}
