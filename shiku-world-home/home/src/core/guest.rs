use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::module::ModuleName;
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::Snowflake;
use crate::persistence_module::models::PersistedGuest;

pub type SessionId = String;
pub type ProviderUserId = String;
pub type ModuleExitSlot = String;
pub type ModuleEnterSlot = String;

#[derive(Debug)]
pub struct Guest {
    pub id: Snowflake,
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
    pub login_data: Option<LoginData>,
    pub is_logged_in: bool,
    pub ws_connection_id: Snowflake,
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
