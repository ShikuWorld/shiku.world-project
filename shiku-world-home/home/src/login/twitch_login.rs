use std::collections::{HashMap, HashSet};

use flume::{unbounded, Receiver};
use jsonwebtoken::{decode, errors::Error as JWTError, Algorithm, DecodingKey, Validation};
use log::{debug, error};
use reqwest::{Client, Error as ReqwestError, Url};
use serde::Deserialize;
use url::ParseError as UrlParseError;

use crate::core::guest::ActorId;
use crate::core::module::{ModuleInputReceiver, ModuleOutputSender};

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
    pub access_token: String,
    #[allow(dead_code)]
    pub expires_in: i32,
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
    pub id: String,
    pub display_name: String,
    pub view_count: i32,
    /*login: String,
    #[serde(rename(deserialize = "type"))]
    kind: String,
    broadcaster_type: String,
    description: String,
    profile_image_url: String,
    offline_image_url: String,*/
    pub created_at: String,
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
