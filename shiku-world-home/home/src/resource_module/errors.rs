use anyhow::Error as AnyhowError;
use serde_json::Error as SerdeJsonError;
use std::io::Error as IOError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SendLoadEventError {
    #[error(transparent)]
    ReadResourceMap(#[from] ReadResourceMapError),
}

#[derive(Error, Debug)]
pub enum SendUnloadEventError {
    #[error("Could not activate resource map for user!")]
    NoActiveResourceMapForUser,
}

#[derive(Error, Debug)]
pub enum ResourceParseError {
    #[error("IOError")]
    IO(#[from] IOError),
    #[error("Could not parse resource.")]
    SerdeParse(#[from] SerdeJsonError),
    #[error("Some unknown error happened!")]
    Misc(#[from] AnyhowError),
    #[error("Could not parse a boolean value!")]
    ParseBool(#[from] ParseBoolError),
    #[error("Could not parse a float value!")]
    ParseFloat(#[from] ParseFloatError),
    #[error("Could not parse a int value!")]
    ParseInt(#[from] ParseIntError),
}

#[derive(Error, Debug)]
pub enum ReadResourceMapError {
    #[error("Could not get resources!")]
    Get,
    #[error("Could not parse resources!")]
    SerdeParse(#[from] SerdeJsonError),
}
