use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::module::ModuleName;
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::Snowflake;
use crate::persistence_module::models::PersistedGuest;
use crate::resource_module::def::GuestId;

pub type SessionId = String;
pub type ProviderUserId = String;
pub type ModuleExitSlot = String;
pub type ModuleEnterSlot = String;

#[derive(Debug)]
pub struct Guest {
    pub id: GuestId,
    pub session_id: SessionId,
    pub current_module: Option<ModuleName>,
    pub current_instance_id: Option<GameInstanceId>,
    pub pending_module_exit: Option<ModuleExitSlot>,
    pub login_data: Option<LoginData>,
    pub ws_connection_id: Option<Snowflake>,
    pub persisted_guest: Option<PersistedGuest>,
}

#[derive(Debug)]
pub struct Admin {
    pub id: Snowflake,
    pub session_id: SessionId,
    pub login_data: Option<LoginData>,
    pub is_logged_in: bool,
    pub ws_connection_id: Option<Snowflake>,
}

pub trait Actors {
    fn get_ws_connection_id(&self) -> Option<Snowflake>;
    fn get_session_id(&self) -> &String;
    fn get_id(&self) -> Snowflake;
    fn set_login_data(&mut self, login_data: LoginData);
    fn set_is_logged_in(&mut self, is_logged_in: bool);
    fn set_ws_connection_id(&mut self, ws_connection_id: Snowflake);
    fn set_session_id(&mut self, session_id: SessionId);
}

impl Actors for Guest {
    fn get_ws_connection_id(&self) -> Option<Snowflake> {
        self.ws_connection_id
    }

    fn get_session_id(&self) -> &String {
        &self.session_id
    }

    fn get_id(&self) -> Snowflake {
        self.id
    }

    fn set_login_data(&mut self, login_data: LoginData) {
        self.login_data = Some(login_data);
    }

    fn set_is_logged_in(&mut self, _is_logged_in: bool) {}

    fn set_ws_connection_id(&mut self, ws_connection_id: Snowflake) {
        self.ws_connection_id = Some(ws_connection_id);
    }

    fn set_session_id(&mut self, session_id: SessionId) {
        self.session_id = session_id;
    }
}

impl Actors for Admin {
    fn get_ws_connection_id(&self) -> Option<Snowflake> {
        self.ws_connection_id
    }

    fn get_session_id(&self) -> &String {
        &self.session_id
    }

    fn get_id(&self) -> Snowflake {
        self.id
    }
    fn set_login_data(&mut self, login_data: LoginData) {
        self.login_data = Some(login_data);
    }

    fn set_is_logged_in(&mut self, is_logged_in: bool) {
        self.is_logged_in = is_logged_in;
    }

    fn set_ws_connection_id(&mut self, ws_connection_id: Snowflake) {
        self.ws_connection_id = Some(ws_connection_id);
    }

    fn set_session_id(&mut self, session_id: SessionId) {
        self.session_id = session_id;
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct OAuth {
    pub access_token: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct LoginData {
    pub provider_user_id: ProviderUserId,
    pub display_name: String,
    pub views: Option<i32>,
    pub provider: LoginProvider,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum LoginProvider {
    Twitch,
    Google,
}
