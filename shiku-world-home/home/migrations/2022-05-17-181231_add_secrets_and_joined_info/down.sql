ALTER TABLE IF EXISTS persisted_guest_states
    DROP COLUMN last_time_joined,
    DROP COLUMN times_joined;

DROP TABLE found_secrets;
