// @generated automatically by Diesel CLI.

diesel::table! {
    found_secrets (id) {
        id -> Int4,
        persisted_guest_state_id -> Int4,
        #[max_length = 64]
        name -> Varchar,
        date -> Timestamp,
    }
}

diesel::table! {
    persisted_guest_states (id) {
        id -> Int4,
        #[max_length = 20]
        twitch_id -> Varchar,
        #[max_length = 15]
        display_name -> Varchar,
        is_observer -> Bool,
        is_tester -> Bool,
        last_time_joined -> Nullable<Timestamp>,
        times_joined -> Int4,
        is_discord_admin -> Bool,
        is_discord_booster -> Bool,
        #[max_length = 20]
        slime_skin_name -> Varchar,
    }
}

diesel::joinable!(found_secrets -> persisted_guest_states (persisted_guest_state_id));

diesel::allow_tables_to_appear_in_same_query!(
    found_secrets,
    persisted_guest_states,
);
