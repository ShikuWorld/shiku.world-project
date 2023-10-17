use log::error;
use thiserror::Error;

use crate::core::blueprint::BlueprintError;
use crate::resource_module::errors::ResourceParseError;

#[derive(Error, Debug)]
pub enum CreateModuleError {
    #[error(transparent)]
    BlueprintError(#[from] BlueprintError),
    #[error(transparent)]
    ResourceParse(#[from] ResourceParseError),
}
