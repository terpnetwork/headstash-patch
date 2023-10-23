use cosmwasm_std::{StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},


    #[error("DuplicateMember: {0}")]
    DuplicateMember(String),

    #[error("NoMemberFound: {0}")]
    NoMemberFound(String),




    #[error("MembersExceeded: {expected} got {actual}")]
    MembersExceeded { expected: u32, actual: u32 },


    #[error("Invalid member limit. min: {min}, max: {max}, got: {got}")]
    InvalidMemberLimit { min: u32, max: u32, got: u32 },

    #[error("Max claim limit per address exceeded")]
    MaxPerAddressLimitExceeded {},


    #[error("InvalidUnitPrice {0} < {1}")]
    InvalidUnitPrice(u128, u128),



    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("UnauthorizedAdmin")]
    UnauthorizedAdmin {},
}
