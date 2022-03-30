use crate::transaction::Kind;
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account is locked")]
    AccountLocked,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction is not found (tx_id: {tx_id:?})")]
    NotFound { tx_id: u32 },

    #[error("Invalid resolution target (tx_id: {tx_id:?}, expected_kind: {expected_kind:?})")]
    InvalidResolutionTarget { tx_id: u32, expected_kind: Kind },

    #[error("Transaction is not being disputed (tx_id: {tx_id:?})")]
    NotDisputed { tx_id: u32 },

    #[error("Transaction is already being disputed (tx_id: {tx_id:?})")]
    AlreadyDisputed { tx_id: u32 },

    #[error("Transaction is already resolved (tx_id: {tx_id:?})")]
    AlreadyResolved { tx_id: u32 },

    #[error("Transaction is already charged back (tx_id: {tx_id:?})")]
    AlreadyChargedBack { tx_id: u32 },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum BalanceError {
    #[error("Insufficient available funds (required: {required:?}, available: {available:?})")]
    InsufficientAvailableFunds {
        available: Decimal,
        required: Decimal,
    },

    #[error("Insufficient held funds (required: {required:?}, held: {held:?})")]
    InsufficientHeldFunds { held: Decimal, required: Decimal },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
