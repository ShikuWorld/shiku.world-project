use thiserror::Error;
use crate::core::blueprint::def::BlueprintError;

#[derive(Error, Debug)]
pub enum CreateWorldError {
    #[error("World already existed.")]
    DidAlreadyExist,
    #[error("Blueprint error during world creation.")]
    BlueprintError(#[from] BlueprintError)
}

#[derive(Debug)]
pub enum DestroyWorldError {
    DidNotExist,
    StillHasInhabitants,
}
