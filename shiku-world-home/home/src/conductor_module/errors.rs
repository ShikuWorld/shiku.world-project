use flume::SendError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

use crate::core::module::GuestToModule;
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
    CouldNotSerializeCommunicationEvent,
}

#[derive(Error, Debug)]
pub enum HandleLoginError {
    #[error("Already logged in.")]
    AlreadyLoggedIn,
    #[error("Could login due to persistence error.")]
    PersistenceError(#[from] PersistenceError),
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
