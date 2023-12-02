#[derive(Debug)]
pub enum CreateWorldError {
    DidAlreadyExist,
}

#[derive(Debug)]
pub enum DestroyWorldError {
    DidNotExist,
    StillHasInhabitants,
}
