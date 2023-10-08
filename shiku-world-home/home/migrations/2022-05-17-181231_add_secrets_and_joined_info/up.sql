ALTER TABLE IF EXISTS persisted_guest_states
    ADD COLUMN last_time_joined timestamp,
    ADD COLUMN times_joined integer NOT NULL DEFAULT 0;

CREATE TABLE found_secrets
(
    id serial NOT NULL,
    persisted_guest_state_id serial NOT NULL,
    name character varying(64) NOT NULL,
    date timestamp NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT secrets_found FOREIGN KEY (persisted_guest_state_id)
        REFERENCES persisted_guest_states (id) MATCH SIMPLE
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT secret_only_found_once_per_guest UNIQUE (persisted_guest_state_id, name)
);
