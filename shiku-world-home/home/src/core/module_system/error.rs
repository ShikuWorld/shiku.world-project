use crate::core::blueprint::def::BlueprintError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateWorldError {
    #[error("World already existed.")]
    DidAlreadyExist,
    #[error("PhysicsPoisenError")]
    PhysicsPoisenError,
    #[error("Could not borrow physics to reset world")]
    BorrowPhysics,
    #[error("Could not find world to reset")]
    CouldNotFindWorld,
    #[error("Blueprint error during world creation.")]
    BlueprintError(#[from] BlueprintError),
}

#[derive(Debug)]
pub enum DestroyWorldError {
    DidNotExist,
    StillHasInhabitants,
}
