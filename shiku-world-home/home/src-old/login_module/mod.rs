use crate::core::guest::{Guest, GuestLoginData, LoginProvider, ModuleEnterSlot};
use crate::core::module::{
    EnterFailedState, EnterSuccessState, GameModule, GameSystemToGuest, GuestEvent,
    GuestStateChange, GuestToModuleEvent, LeaveFailedState, LeaveSuccessState, ModuleInputReceiver,
    ModuleName, ModuleOutputSender, ModuleState, ModuleToSystemEvent, SystemModule,
    SystemToModuleEvent, ToastAlertLevel,
};
use chrono::prelude::*;
use flume::{unbounded, Receiver};
use jsonwebtoken::{decode, errors::Error as JWTError, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::core::module::GameSystemToGuestEvent;
use crate::resource_module::def::{GuestId, ResourceFile, ResourceModule};
use crate::resource_module::errors::ResourceParseError;
use log::{debug, error};

use reqwest::blocking::Client as BlockingClient;
use reqwest::{Client, Error as ReqwestError, Url};
use url::ParseError as UrlParseError;

pub struct LoginModule {
    current_guests: HashSet<GuestId>,
    twitch_ext_access_token: String,
    twitch_login_process_running: HashMap<GuestId, TwitchApiLogin>,
    input_receiver: ModuleInputReceiver,
    output_sender: ModuleOutputSender,
}

pub struct TwitchApiLogin {
    receiver: Receiver<Result<TwitchUserResponseData, TwitchApiError>>,
    pub result: Option<Result<TwitchUserResponseData, TwitchApiError>>,
}

#[derive(Debug)]
pub enum TwitchApiError {
    ReqwestError(ReqwestError),
    UrlParseError(UrlParseError),
    JWTError(JWTError),
    InvalidResponse(String),
    NoAuthCodeProvided,
}

impl From<ReqwestError> for TwitchApiError {
    fn from(err: ReqwestError) -> Self {
        TwitchApiError::ReqwestError(err)
    }
}

impl From<UrlParseError> for TwitchApiError {
    fn from(err: UrlParseError) -> Self {
        TwitchApiError::UrlParseError(err)
    }
}

impl From<JWTError> for TwitchApiError {
    fn from(err: JWTError) -> Self {
        TwitchApiError::JWTError(err)
    }
}

#[derive(Deserialize, Debug)]
pub struct TwitchExtensionOauthTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: i32,
    /*scope: Option<Vec<String>>,
    token_type: String,*/
}

#[derive(Deserialize, Debug)]
pub struct TwitchOauthTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: String,
    #[allow(dead_code)]
    expires_in: i32,
    /*scope: Option<Vec<String>>,
    token_type: String,*/
}

#[derive(Deserialize)]
pub struct TwitchUserResponse {
    data: Vec<TwitchUserResponseData>,
}

#[derive(Deserialize, Debug)]
pub struct TwitchUserResponseData {
    id: String,
    display_name: String,
    view_count: i32,
    /*login: String,
    #[serde(rename(deserialize = "type"))]
    kind: String,
    broadcaster_type: String,
    description: String,
    profile_image_url: String,
    offline_image_url: String,*/
    created_at: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TwitchJWTClaims {
    exp: i32,
    opaque_user_id: String,
    role: String,
    channel_id: String,
    user_id: String,
    iat: Option<i32>,
}

impl TwitchApiLogin {
    async fn _login(
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        extension_secret: &str,
        auth_code_option: Option<String>,
        jwt_token_option: Option<String>,
        extension_access_token: String,
    ) -> Result<TwitchUserResponseData, TwitchApiError> {
        let client = Client::new();

        let (access_token, user_id_option) = if let Some(jwt_token) = jwt_token_option {
            let token = decode::<TwitchJWTClaims>(
                &jwt_token,
                &DecodingKey::from_base64_secret(extension_secret)?,
                &Validation::new(Algorithm::HS256),
            )?;

            (extension_access_token, Some(token.claims.user_id))
        } else if let Some(auth_code) = auth_code_option {
            let url = Url::parse_with_params(
                "https://id.twitch.tv/oauth2/token",
                &[
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                    ("grant_type", "authorization_code"),
                    ("redirect_uri", redirect_uri),
                    ("code", &auth_code),
                ],
            )?;

            let oauth_response_as_json = client
                .post(url)
                .send()
                .await?
                .json::<TwitchOauthTokenResponse>()
                .await?;

            (oauth_response_as_json.access_token, None)
        } else {
            return Err(TwitchApiError::NoAuthCodeProvided);
        };

        let client = Client::new();
        let mut request = client
            .get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", access_token.clone()))
            .header("Client-Id", client_id);

        if let Some(user_id) = user_id_option {
            request = request.query(&[("id", user_id)]);
        }

        let response = request.send().await?.json::<TwitchUserResponse>().await?;

        if response.data.len() != 1 {
            return Err(TwitchApiError::InvalidResponse(
                "Didn't get exactly one user data, weird.".to_string(),
            ));
        }

        if let Some(response_data) = response.data.into_iter().next() {
            Ok(response_data)
        } else {
            Err(TwitchApiError::InvalidResponse(
                "What the hell, this is impossible!?!".to_string(),
            ))
        }
    }

    pub fn login(
        auth_code: Option<String>,
        jwt_token: Option<String>,
        extension_access_token: String,
    ) -> TwitchApiLogin {
        let (sender, receiver) = unbounded();
        tokio::spawn(async move {
            let client_id = "uchpfk924k24ozzra8f6i7bpthn33r";
            let client_secret = "0t6mrb7w4j6o8l5dq51rncmt8u91uu";
            let extension_secret = "2Z0pxwocuMUO6IPSfQVA5xYK/ZgDat164xjSONkG6nI=";
            let redirect_uri = "https://localhost:8080";
            if let Err(err) = match Self::_login(
                client_id,
                client_secret,
                redirect_uri,
                extension_secret,
                auth_code,
                jwt_token,
                extension_access_token,
            )
            .await
            {
                Ok(twitch_user_data) => sender.send(Ok(twitch_user_data)),
                Err(err) => {
                    debug!("Sending twitch api error");
                    sender.send(Err(err))
                }
            } {
                error!("Could not send twitch login data to be polled?! {:?}", err);
            }
        });

        TwitchApiLogin {
            receiver,
            result: None,
        }
    }

    pub fn poll_result(&mut self) -> bool {
        for received_result in self.receiver.try_iter() {
            self.result = Some(received_result);
        }

        self.result.is_some()
    }
}

impl SystemModule for LoginModule {
    fn module_name(&self) -> ModuleName {
        Self::module_name()
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

impl GameModule for LoginModule {
    fn get_base_resource_file(&self) -> ResourceFile {
        ResourceFile {
            module_name: self.module_name(),
            resources: Vec::new(),
        }
    }

    fn get_resource_json(&self) -> String {
        String::from_utf8_lossy(include_bytes!("login.resources.json")).to_string()
    }

    fn update(&mut self) {
        self.process_input_events();
        self.process_system_input_events();
        self.process_running_logins();
    }

    fn try_enter(
        &mut self,
        guest: &Guest,
        module_enter_slot: &ModuleEnterSlot,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        if self.current_guests.contains(&guest.id) {
            return Err(EnterFailedState::AlreadyEntered);
        }

        self.current_guests.insert(guest.id);
        debug!("Guest entered login.");

        Ok(EnterSuccessState::Entered)
    }

    fn try_leave(&mut self, guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        if self.current_guests.contains(&guest.id) {
            debug!("Guest left login.");
            self.current_guests.remove(&guest.id);
            return Ok(LeaveSuccessState::Left);
        }

        Err(LeaveFailedState::NotInModule)
    }
}

pub struct LoginModuleExitSlots {
    pub exit: &'static str,
}

pub struct LoginModuleEnterSlots {
    pub enter: &'static str,
}

impl LoginModule {
    pub const EXIT_SLOTS: LoginModuleExitSlots = LoginModuleExitSlots { exit: "login_exit" };

    pub const ENTER_SLOTS: LoginModuleEnterSlots = LoginModuleEnterSlots {
        enter: "login_enter",
    };

    pub fn module_name() -> ModuleName {
        "LoginModule".to_string()
    }

    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> LoginModule {
        let twitch_ext_access_token = tokio::task::block_in_place(|| {
            let client_id = "uchpfk924k24ozzra8f6i7bpthn33r";
            let client_secret = "0t6mrb7w4j6o8l5dq51rncmt8u91uu";

            let client = BlockingClient::new();
            let url = Url::parse_with_params(
                "https://id.twitch.tv/oauth2/token",
                &[
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                    ("grant_type", "client_credentials"),
                ],
            )
            .unwrap();

            let oauth_response_as_json = client
                .post(url)
                .send()
                .unwrap()
                .json::<TwitchExtensionOauthTokenResponse>()
                .unwrap();

            oauth_response_as_json.access_token
        });

        LoginModule {
            twitch_ext_access_token,
            current_guests: HashSet::new(),
            twitch_login_process_running: HashMap::new(),
            input_receiver,
            output_sender,
        }
    }

    fn process_running_logins(&mut self) {
        let finished_logins: Vec<GuestId> = self
            .twitch_login_process_running
            .iter_mut()
            .filter_map(|(guest_id, process)| {
                if process.poll_result() {
                    Some(*guest_id)
                } else {
                    None
                }
            })
            .collect();

        for guest_id in finished_logins {
            if let Some(login) = self.twitch_login_process_running.remove(&guest_id) {
                if let Some(result) = login.result {
                    match result {
                        Ok(user_response_data) => {
                            if let Ok(parsed_date) =
                                DateTime::parse_from_rfc3339(user_response_data.created_at.as_str())
                            {
                                let days_since_account_creation =
                                    Utc::now().signed_duration_since(parsed_date).num_days();

                                if days_since_account_creation < 0 {
                                    if let Err(err) = self.output_sender.game_system_to_guest_sender.send(GameSystemToGuest {
                                        guest_id: guest_id.clone(),
                                        event_type: GameSystemToGuestEvent::UpdateDataStore(format!(
                                            "{{\"login_error\": \"{}\" }}",
                                            "Your account has not existed for more than 7 days, ask Shiku to unlock you manually!"
                                        )),
                                    }) {
                                        error!("Could not send error message to guest. {:?}", err);
                                    }
                                    if let Err(err) = self
                                        .output_sender
                                        .module_to_system_sender
                                        .send(GuestEvent {
                                            guest_id,
                                            event_type: ModuleToSystemEvent::LoginFailed,
                                        })
                                    {
                                        error!("Could not send login failed to guest. {:?}", err);
                                    }
                                } else if let Err(err) =
                                    self.output_sender.module_to_system_sender.send(GuestEvent {
                                        guest_id,
                                        event_type: ModuleToSystemEvent::GuestStateChange(
                                            GuestStateChange::LoginAndTargetModule(
                                                GuestLoginData {
                                                    provider_user_id: user_response_data.id,
                                                    display_name: user_response_data.display_name,
                                                    views: Some(user_response_data.view_count),
                                                    provider: LoginProvider::Twitch,
                                                },
                                                LoginModule::EXIT_SLOTS.exit.into(),
                                            ),
                                        ),
                                    })
                                {
                                    error!("{:?}", err);
                                }
                            }
                        }
                        Err(err) => {
                            error!("Could not login guest with twitch api, hm. {:?}", err);
                            if let Err(err) = self.output_sender.game_system_to_guest_sender.send(
                                GameSystemToGuest {
                                    guest_id: guest_id.clone(),
                                    event_type: GameSystemToGuestEvent::UpdateDataStore(format!(
                                    "{{\"login_error\": \"{}\" }}",
                                    "Could not login, please try a different method or ask Shiku."
                                )),
                                },
                            ) {
                                error!("Could not send error message to guest. {:?}", err);
                            }
                            if let Err(err) =
                                self.output_sender.module_to_system_sender.send(GuestEvent {
                                    guest_id,
                                    event_type: ModuleToSystemEvent::LoginFailed,
                                })
                            {
                                error!("Could not send login failed to guest. {:?}", err);
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_input_events(&mut self) {
        for GuestEvent {
            guest_id,
            event_type,
        } in self.input_receiver.guest_to_module_receiver.drain()
        {
            match event_type {
                GuestToModuleEvent::ResourcesLoaded(module_name) => {
                    debug!("Resources for {} finished loading from user", module_name);
                    if let Err(err) =
                        self.output_sender
                            .game_system_to_guest_sender
                            .send(GuestEvent {
                                guest_id,
                                event_type: GameSystemToGuestEvent::OpenMenu(
                                    "LoginModuleMenu".to_string(),
                                ),
                            })
                    {
                        error!("{:?}", err);
                    }
                }
                GuestToModuleEvent::ProviderLoggedIn(provider_logged_in) => {
                    self.current_guests.insert(guest_id);
                    match provider_logged_in.login_provider {
                        LoginProvider::Twitch => {
                            self.twitch_login_process_running.insert(
                                guest_id,
                                TwitchApiLogin::login(
                                    provider_logged_in.auth_code,
                                    provider_logged_in.access_token,
                                    self.twitch_ext_access_token.clone(),
                                ),
                            );
                        }
                        LoginProvider::Google => {
                            debug!("Implement google provider!");
                        }
                    }
                }
                GuestToModuleEvent::WantToChangeModule(_) => {}
                GuestToModuleEvent::ControlInput(_) => {}
                GuestToModuleEvent::Ping => {}
            }
        }
    }

    fn process_system_input_events(&mut self) {
        for GuestEvent {
            guest_id,
            event_type,
        } in self.input_receiver.system_to_module_receiver.drain()
        {
            match event_type {
                SystemToModuleEvent::Disconnected => {}
                SystemToModuleEvent::Reconnected => {}
                SystemToModuleEvent::AlreadyLoggedIn => {
                    debug!("Guest was already logged in.");
                    if let Err(err) = self.output_sender.game_system_to_guest_sender.send(GameSystemToGuest {
                        guest_id: guest_id.clone(),
                        event_type: GameSystemToGuestEvent::UpdateDataStore(format!(
                            "{{\"login_error\": \"{}\" }}",
                            "You are already logged in with this account, please check where you are logged in."
                        )),
                    }) {
                        error!("Could not send error message to guest. {:?}", err);
                    }

                    if let Err(err) = self.output_sender.module_to_system_sender.send(GuestEvent {
                        guest_id,
                        event_type: ModuleToSystemEvent::LoginFailed,
                    }) {
                        error!("Could not send login failed to guest. {:?}", err);
                    }
                }
            }
        }
    }
}
