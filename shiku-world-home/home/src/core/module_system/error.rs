use crate::core::blueprint::def::BlueprintError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateWorldError {
    #[error("World already existed.")]
    DidAlreadyExist,
    #[error("Blueprint error during world creation.")]
    BlueprintError(#[from] BlueprintError),
    #[error("PhysicsPoisenError")]
    PhysicsPoisenError,
}

#[derive(Debug)]
pub enum DestroyWorldError {
    DidNotExist,
    StillHasInhabitants,
}
