use ink::prelude::string::String;

/// PSP-22 Error types following the standard
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[allow(clippy::cast_possible_truncation)]
pub enum PSP22Error {
    /// Insufficient balance for transfer
    InsufficientBalance,
    /// Insufficient allowance for transfer_from
    InsufficientAllowance,
    /// Custom error with message
    Custom(String),
}
