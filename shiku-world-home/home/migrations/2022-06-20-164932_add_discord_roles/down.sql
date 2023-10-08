ALTER TABLE IF EXISTS persisted_guest_states
    DROP COLUMN is_discord_admin,
    DROP COLUMN is_discord_booster;
