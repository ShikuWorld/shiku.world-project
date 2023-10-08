use super::schema::{found_secrets, persisted_guest_states};
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "persisted_guest_states"]
pub struct PersistedGuestState {
    pub id: i32,
    pub twitch_id: String,
    pub display_name: String,
    pub is_observer: bool,
    pub is_tester: bool,
    pub last_time_joined: Option<NaiveDateTime>,
    pub times_joined: i32,
    pub is_discord_admin: bool,
    pub is_discord_booster: bool,
    pub slime_skin_name: String,
}

#[derive(Debug)]
pub struct PersistedGuest {
    pub info: PersistedGuestState,
    pub secrets_found: Vec<FoundSecret>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(PersistedGuestState)]
#[table_name = "found_secrets"]
pub struct FoundSecret {
    pub id: i32,
    pub persisted_guest_state_id: i32,
    pub name: String,
    pub date: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "persisted_guest_states"]
pub struct NewPersistedGuestState {
    pub twitch_id: String,
    pub display_name: String,
    pub is_observer: bool,
    pub is_tester: bool,
}

#[derive(AsChangeset)]
#[table_name = "persisted_guest_states"]
pub struct UpdatePersistedGuestState {
    pub id: i32,
    pub is_observer: Option<bool>,
    pub is_tester: Option<bool>,
    pub last_time_joined: Option<NaiveDateTime>,
    pub times_joined: Option<i32>,
}

#[derive(Insertable)]
#[table_name = "found_secrets"]
pub struct NewFoundSecret {
    pub persisted_guest_state_id: i32,
    pub name: String,
    pub date: NaiveDateTime,
}
