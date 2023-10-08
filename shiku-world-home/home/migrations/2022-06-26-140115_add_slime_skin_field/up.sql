ALTER TABLE IF EXISTS persisted_guest_states
    ADD COLUMN slime_skin_name VARCHAR(20) NOT NULL DEFAULT '';
