use flume::SendError;
use serde_json::Error as SerdeJsonError;

use crate::core::module::GuestToModule;
use crate::core::Snowflake;
use crate::persistence_module::PersistenceError;

#[derive(Debug)]
pub enum SendEventToModuleError {
    EventSendError(SendError<GuestToModule>),
    SerdeParseError(SerdeJsonError),
}

#[derive(Debug)]
pub enum ProcessModuleEventError {
    GuestNotFound,
    PersistenceError(PersistenceError),
    GuestAlreadyLoggedIn(Snowflake),
    CouldNotSerializeCommunicationEvent,
}

#[derive(Debug)]
pub enum ProcessGameEventError {
    CouldNotSerializePosition,
}

impl From<PersistenceError> for ProcessModuleEventError {
    fn from(err: PersistenceError) -> Self {
        ProcessModuleEventError::PersistenceError(err)
    }
}

impl From<SerdeJsonError> for SendEventToModuleError {
    fn from(err: SerdeJsonError) -> Self {
        SendEventToModuleError::SerdeParseError(err)
    }
}

impl From<SendError<GuestToModule>> for SendEventToModuleError {
    fn from(err: SendError<GuestToModule>) -> Self {
        SendEventToModuleError::EventSendError(err)
    }
}
