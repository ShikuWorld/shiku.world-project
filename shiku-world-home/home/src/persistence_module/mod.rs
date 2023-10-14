use std::env;
use std::fmt::{Display, Formatter};

use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error as DieselResultError;
use thiserror::Error;

use schema::found_secrets;
use schema::persisted_guest_states;

use crate::core::module::{ModuleName, ModuleState};
use crate::persistence_module::models::{
    FoundSecret, NewFoundSecret, NewPersistedGuestState, PersistedGuest, PersistedGuestState,
    UpdatePersistedGuestState,
};
use crate::SystemModule;

pub mod models;
pub mod schema;

#[derive(Error, Debug)]
pub enum PersistenceError {
    DieselResultError(#[from] DieselResultError),
    R2D2Error(String),
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::DieselResultError(err) => {
                write!(f, "DieselResultError {:?}", err)
            }
            PersistenceError::R2D2Error(err) => {
                write!(f, "R2D2Error {:?}", err)
            }
        }
    }
}

pub struct PersistenceModule {
    connection_pool: Pool<ConnectionManager<PgConnection>>,
}

impl PersistenceModule {
    pub fn new() -> PersistenceModule {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        PersistenceModule {
            connection_pool: Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("Could not establish connection pool"),
        }
    }

    fn get_connection_from_pool(
        connection_pool: &Pool<ConnectionManager<PgConnection>>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, PersistenceError> {
        match connection_pool.get() {
            Ok(connection) => Ok(connection),
            Err(err) => Err(PersistenceError::R2D2Error(err.to_string())),
        }
    }

    pub fn lazy_get_persisted_guest_by_provider_id(
        &self,
        provider_id: &String,
        display_name: &String,
    ) -> Result<PersistedGuest, PersistenceError> {
        let mut connection = Self::get_connection_from_pool(&self.connection_pool)?;

        let persisted_guest_state = Self::lazy_get_persisted_guest_state_by_provider_id(
            &mut connection,
            provider_id,
            display_name,
        )?;

        let secrets =
            FoundSecret::belonging_to(&persisted_guest_state).get_results(&mut connection)?;

        Ok(PersistedGuest {
            info: persisted_guest_state,
            secrets_found: secrets,
        })
    }

    pub fn update_persisted_guest_state(
        &self,
        update_persisted_guest_state: UpdatePersistedGuestState,
    ) -> Result<usize, PersistenceError> {
        let mut connection = Self::get_connection_from_pool(&self.connection_pool)?;

        let target = persisted_guest_states::dsl::persisted_guest_states
            .filter(persisted_guest_states::dsl::id.eq(update_persisted_guest_state.id));

        Ok(diesel::update(target)
            .set(&update_persisted_guest_state)
            .execute(&mut connection)?)
    }

    pub fn lazy_get_persisted_guest_state_by_provider_id(
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        provider_id: &String,
        display_name: &String,
    ) -> Result<PersistedGuestState, PersistenceError> {
        let result = persisted_guest_states::dsl::persisted_guest_states
            .filter(persisted_guest_states::dsl::twitch_id.eq(provider_id))
            .first::<PersistedGuestState>(connection);

        match result {
            Ok(persisted_guest_state) => Ok(persisted_guest_state),
            Err(DieselResultError::NotFound) => match Self::create_guest_persisted_state(
                connection,
                display_name.clone(),
                provider_id.clone(),
                false,
                false,
            ) {
                Ok(persisted_guest_state) => Ok(persisted_guest_state),
                Err(err) => Err(PersistenceError::DieselResultError(err)),
            },
            Err(err) => Err(PersistenceError::DieselResultError(err)),
        }
    }

    pub fn create_guest_persisted_state(
        conn: &mut PgConnection,
        display_name: String,
        twitch_id: String,
        is_observer: bool,
        is_tester: bool,
    ) -> Result<PersistedGuestState, DieselResultError> {
        let new_persisted_guest_state = NewPersistedGuestState {
            display_name,
            twitch_id,
            is_observer,
            is_tester,
        };

        diesel::insert_into(persisted_guest_states::table)
            .values(&new_persisted_guest_state)
            .get_result(conn)
    }

    pub fn add_secret_found(
        &self,
        name: String,
        persisted_guest_state_id: i32,
    ) -> Result<FoundSecret, PersistenceError> {
        let mut conn = Self::get_connection_from_pool(&self.connection_pool)?;

        let new_secret_found = NewFoundSecret {
            name,
            date: Utc::now().naive_utc(),
            persisted_guest_state_id,
        };

        Ok(diesel::insert_into(found_secrets::table)
            .values(&new_secret_found)
            .get_result(&mut conn)?)
    }
}

impl SystemModule for PersistenceModule {
    fn module_name(&self) -> ModuleName {
        "WebServerModule".to_string()
    }

    fn status(&self) -> &ModuleState {
        todo!()
    }

    fn start(&mut self) {
        todo!()
    }

    fn shutdown(&mut self) {
        todo!()
    }
}
