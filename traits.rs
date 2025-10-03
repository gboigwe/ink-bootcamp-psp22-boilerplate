use ink::prelude::{string::String, vec::Vec};

use crate::data::PSP22Error;

#[ink::trait_definition]
pub trait PSP22 {
    /// Returns the total token supply.
    #[ink(message)]
    fn total_supply(&self) -> u128;

    /// Returns the account balance for the specified `owner`.
    ///
    /// Returns `0` if the account is non-existent.
    #[ink(message)]
    fn balance_of(&self, owner: ink::primitives::AccountId) -> u128;

    /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
    ///
    /// Returns `0` if no allowance has been set.
    #[ink(message)]
    fn allowance(&self, owner: ink::primitives::AccountId, spender: ink::primitives::AccountId) -> u128;

    /// Transfers `value` amount of tokens from the caller's account to account `to`
    /// with additional `data` in unspecified format.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if the caller and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    #[ink(message)]
    fn transfer(&mut self, to: ink::primitives::AccountId, value: u128, data: Vec<u8>) -> Result<(), PSP22Error>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`
    /// with additional `data` in unspecified format.
    ///
    /// If `from` and the caller are different addresses, the caller must be allowed
    /// by `from` to spend at least `value` tokens.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if `from` and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// If `from` and the caller are different addresses, a successful transfer results
    /// in decreased allowance by `from` to the caller and an `Approval` event with
    /// the new allowance amount is emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the balance of the account `from`.
    ///
    /// Reverts with `InsufficientAllowance` if `from` and the caller are different addresses and
    /// the `value` exceeds the allowance granted by `from` to the caller.
    ///
    /// If conditions for both `InsufficientBalance` and `InsufficientAllowance` errors are met,
    /// reverts with `InsufficientAllowance`.
    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: ink::primitives::AccountId,
        to: ink::primitives::AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the total amount of `value`.
    ///
    /// Successive calls of this method overwrite previous values.
    ///
    /// # Events
    ///
    /// An `Approval` event is emitted.
    ///
    /// No-op if the caller and `spender` is the same address, returns success and no events are emitted.
    #[ink(message)]
    fn approve(&mut self, spender: ink::primitives::AccountId, value: u128) -> Result<(), PSP22Error>;

    /// Increases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    #[ink(message)]
    fn increase_allowance(
        &mut self,
        spender: ink::primitives::AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error>;

    /// Decreases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientAllowance` if `spender` and the caller are different addresses and
    /// the `delta-value` exceeds the allowance granted by the caller to `spender`.
    #[ink(message)]
    fn decrease_allowance(
        &mut self,
        spender: ink::primitives::AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
pub trait PSP22Metadata {
    /// Returns the token name.
    #[ink(message)]
    fn name(&self) -> Option<String>;
    /// Returns the token symbol.
    #[ink(message)]
    fn symbol(&self) -> Option<String>;
    /// Returns the token decimals.
    #[ink(message)]
    fn decimals(&self) -> u8;
}

#[ink::trait_definition]
pub trait PSP22Burnable {
    /// Burns `value` tokens from the senders account.
    ///
    /// The selector for this message is `0x7a9da510` (first 4 bytes of `blake2b_256("PSP22Burnable::burn")`).
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` recipient.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    #[ink(message)]
    fn burn(&mut self, value: u128) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
pub trait PSP22Mintable {
    /// Mints `value` tokens to the senders account.
    ///
    /// The selector for this message is `0xfc3c75d4` (first 4 bytes of `blake2b_256("PSP22Mintable::mint")`).
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted with `None` sender.
    ///
    /// No-op if `value` is zero, returns success and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `Custom (max supply exceeded)` if the total supply increased by
    /// `value` exceeds maximal value of `u128` type.
    #[ink(message)]
    fn mint(&mut self, value: u128) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
#[allow(dead_code)]
pub trait PSP22Permit {
    /// Allows anyone to call approve on behalf of `owner` if the signature is valid.
    ///
    /// Must provide the v, r, s parts of the signature.
    #[ink(message)]
    #[allow(clippy::too_many_arguments)]
    fn permit(&mut self, owner: ink::primitives::AccountId, spender: ink::primitives::AccountId, value: u128, deadline: u64, v: u8, r: [u8; 32], s: [u8; 32]) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn nonces(&self, owner: ink::primitives::AccountId) -> u128;
}