use cosmwasm_std::{Addr, StdError};
use cw_utils::{self, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Contract has no funds")]
    NoFunds {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Unauthorized admin, sender is {sender}")]
    Unauthorized { sender: Addr },

    #[error("Reply error")]
    ReplyOnSuccess {},


    #[error("Address {address} is not eligible")]
    AddressNotEligible { address: String },

    #[error("Address {address} has already claimed the headstash allocation")]
    ClaimCountReached { address: String },

    #[error("Plaintext message must contain `{{wallet}}` string")]
    PlaintextMsgNoWallet {},

    #[error("Plaintext message is too long")]
    PlaintextTooLong {},

    #[error("Not Found")]
    CwGoopAddressMissing {},
}
