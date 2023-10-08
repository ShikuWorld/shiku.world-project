use anyhow::Error as AnyhowError;
use flume::TrySendError;
use serde_json::Error as SerdeJsonError;
use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

use crate::core::module::GuestEvent;
use crate::resource_module::def::ResourceEvent;

#[derive(Debug)]
pub enum SendLoadEventError {
    TrySendError(TrySendError<GuestEvent<ResourceEvent>>),
    MapGetError,
    SerdeParseError(SerdeJsonError),
}

#[derive(Debug)]
pub enum SendUnloadEventError {
    NoActiveResourceMapForUser,
}

#[derive(Debug)]
pub enum ResourceParseError {
    IOError(IOError),
    SerdeParseError(SerdeJsonError),
    XMLParseError(String),
    MiscError(AnyhowError),
    ParseBoolError(ParseBoolError),
}

#[derive(Debug)]
pub enum ReadResourceMapError {
    Get,
    SerdeParseError(SerdeJsonError),
}

impl From<ParseBoolError> for ResourceParseError {
    fn from(err: ParseBoolError) -> Self {
        ResourceParseError::ParseBoolError(err)
    }
}

impl From<SerdeJsonError> for SendLoadEventError {
    fn from(err: SerdeJsonError) -> Self {
        SendLoadEventError::SerdeParseError(err)
    }
}

impl From<TrySendError<GuestEvent<ResourceEvent>>> for SendLoadEventError {
    fn from(err: TrySendError<GuestEvent<ResourceEvent>>) -> Self {
        SendLoadEventError::TrySendError(err)
    }
}

impl From<SerdeJsonError> for ReadResourceMapError {
    fn from(err: SerdeJsonError) -> Self {
        ReadResourceMapError::SerdeParseError(err)
    }
}

impl From<ReadResourceMapError> for SendLoadEventError {
    fn from(err: ReadResourceMapError) -> Self {
        match err {
            ReadResourceMapError::Get => SendLoadEventError::MapGetError,
            ReadResourceMapError::SerdeParseError(err) => SendLoadEventError::SerdeParseError(err),
        }
    }
}

impl From<AnyhowError> for ResourceParseError {
    fn from(err: AnyhowError) -> Self {
        ResourceParseError::MiscError(err)
    }
}

impl From<IOError> for ResourceParseError {
    fn from(err: IOError) -> Self {
        ResourceParseError::IOError(err)
    }
}

impl From<SerdeJsonError> for ResourceParseError {
    fn from(err: SerdeJsonError) -> Self {
        ResourceParseError::SerdeParseError(err)
    }
}

impl From<ParseIntError> for ResourceParseError {
    fn from(err: ParseIntError) -> Self {
        ResourceParseError::XMLParseError(err.to_string())
    }
}

impl From<ParseFloatError> for ResourceParseError {
    fn from(err: ParseFloatError) -> Self {
        ResourceParseError::XMLParseError(err.to_string())
    }
}
