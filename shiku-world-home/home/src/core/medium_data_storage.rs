use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct MediumDataStorage {
    pub current_guest_info: Option<MediumDataStorageGuestInfo>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct MediumDataStorageGuestInfo {
    pub guests_online: i32,
    pub guest_name: String,
    pub times_joined: i32,
    pub secrets_found_count: i32,
    pub secrets_found_map: HashMap<String, SecretFoundEntry>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct SecretFoundEntry {
    pub name: String,
    pub date: i64,
}
