use std::num::TryFromIntError;

use cosmwasm_std::{Response, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

pub type ContractResult = Result<Response, ContractError>;
pub type CustomResult<T> = Result<T, ContractError>;

/// This enum describes hub contract errors
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    TryFromInt(#[from] TryFromIntError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("not supported")]
    NotSupported,
}
