use ink::prelude::string::String;
use scale::{Encode, Decode};

/// PSP-22 Error types following the standard
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
#[allow(clippy::cast_possible_truncation)]
pub enum PSP22Error {
    /// Insufficient balance for transfer
    InsufficientBalance,
    /// Insufficient allowance for transfer_from
    InsufficientAllowance,
    /// Custom error with message
    Custom(String),
}
