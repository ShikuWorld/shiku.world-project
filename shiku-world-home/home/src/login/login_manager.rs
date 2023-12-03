use std::collections::HashMap;

use chrono::{DateTime, Utc};
use log::debug;
use reqwest::blocking::Client as BlockingClient;
use reqwest::Url;

use crate::core::guest::ActorId;
use crate::core::guest::{LoginData, LoginProvider};
use crate::core::module::ProviderLoggedIn;
use crate::login::twitch_login::{
    TwitchApiError, TwitchApiLogin, TwitchExtensionOauthTokenResponse,
};

pub struct LoginManager {
    twitch_ext_access_token: String,
    twitch_login_process_running: HashMap<ActorId, TwitchApiLogin>,
    twitch_admin_login_process_running: HashMap<ActorId, TwitchApiLogin>,
    finished_logins: Vec<ActorId>,
}

pub enum LoginError {
    UserDidNotExistLongEnough(ActorId, i64),
    TwitchApiError(ActorId, TwitchApiError),
}

const MIN_DAYS_SINCE_ACCOUNT_CREATION: i64 = 0;

impl LoginManager {
    pub fn new() -> LoginManager {
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

        LoginManager {
            finished_logins: Vec::new(),
            twitch_ext_access_token,
            twitch_login_process_running: HashMap::new(),
            twitch_admin_login_process_running: HashMap::new(),
        }
    }

    pub fn process_running_logins<F>(&mut self, mut callback: F)
    where
        F: FnMut(Result<(ActorId, LoginData), LoginError>),
    {
        self.finished_logins
            .extend(self.twitch_login_process_running.iter_mut().filter_map(
                |(guest_id, process)| {
                    if process.poll_result() {
                        Some(*guest_id)
                    } else {
                        None
                    }
                },
            ));

        for guest_id in self.finished_logins.drain(..) {
            if let Some(login) = self.twitch_login_process_running.remove(&guest_id) {
                if let Some(result) = login.result {
                    match result {
                        Ok(user_response_data) => {
                            if let Ok(parsed_date) =
                                DateTime::parse_from_rfc3339(user_response_data.created_at.as_str())
                            {
                                let days_since_account_creation =
                                    Utc::now().signed_duration_since(parsed_date).num_days();

                                if days_since_account_creation < MIN_DAYS_SINCE_ACCOUNT_CREATION {
                                    callback(Err(LoginError::UserDidNotExistLongEnough(
                                        guest_id,
                                        MIN_DAYS_SINCE_ACCOUNT_CREATION,
                                    )));
                                } else {
                                    callback(Ok((
                                        guest_id,
                                        LoginData {
                                            provider_user_id: user_response_data.id,
                                            display_name: user_response_data.display_name,
                                            views: Some(user_response_data.view_count),
                                            provider: LoginProvider::Twitch,
                                        },
                                    )));
                                }
                            }
                        }
                        Err(err) => {
                            callback(Err(LoginError::TwitchApiError(guest_id, err)));
                        }
                    }
                }
            }
        }
    }
    pub fn add_provider_login(&mut self, guest_id: ActorId, provider_logged_in: ProviderLoggedIn) {
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
}

/*
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
                                               LoginData {
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
                           } else {
                               error!("Could not parse user_response_data.created_at");
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
*/
