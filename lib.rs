#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod traits;

#[ink::contract]
mod psp_coin {
    use ink::{storage::Mapping, prelude::vec::Vec, prelude::string::String};

    use crate::{
        data::PSP22Error,
        traits::{PSP22Burnable, PSP22Metadata, PSP22Mintable, PSP22},
    };

    /// Storage structure for the PSP-22 token
    #[ink(storage)]
    pub struct PspCoin {
        /// Total supply of tokens
        total_supply: u128,
        /// Mapping from account to token balance
        balances: Mapping<AccountId, u128>,
        /// Nested mapping for allowances (owner, spender) -> amount
        allowances: Mapping<(AccountId, AccountId), u128>,
        /// Token metadata (name, symbol, decimals)
        metadata: (String, String, u8),
    }

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Option<AccountId>,
        #[ink(topic)]
        pub to: Option<AccountId>,
        pub value: u128,
    }

    /// Event emitted when an approval is granted
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub spender: AccountId,
        pub value: u128,
    }

    impl PspCoin {
        /// Constructor that initializes with zero supply
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                total_supply: 0,
                balances: Mapping::default(),
                allowances: Mapping::default(),
                metadata: (
                    String::from("PSP Coin"),
                    String::from("PSP"),
                    18,
                ),
            }
        }

        /// Constructor that initializes with a specific supply
        #[ink(constructor)]
        pub fn new_with_supply(initial_supply: u128) -> Self {
            let caller = Self::env().caller();

            let mut balances = Mapping::default();
            balances.insert(caller, &initial_supply);

            Self {
                total_supply: initial_supply,
                balances,
                allowances: Mapping::default(),
                metadata: (
                    String::from("PSP Coin"),
                    String::from("PSP"),
                    18,
                ),
            }
        }
    }

    impl PSP22 for PspCoin {
        /// Returns the total token supply
        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.total_supply
        }

        /// Returns the balance of the specified owner
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.balances.get(owner).unwrap_or(0)
        }

        /// Returns the allowance granted by owner to spender
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        /// Transfer tokens from caller to recipient
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: u128, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let from = self.env().caller();

            // No-op if transferring to self or value is zero
            if from == to || value == 0 {
                return Ok(());
            }

            // Check caller's balance
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            // Update balances with overflow protection
            let new_from_balance = from_balance
                .checked_sub(value)
                .ok_or(PSP22Error::InsufficientBalance)?;

            let to_balance = self.balance_of(to);
            let new_to_balance = to_balance
                .checked_add(value)
                .ok_or(PSP22Error::Custom(String::from("Overflow")))?;

            self.balances.insert(from, &new_from_balance);
            self.balances.insert(to, &new_to_balance);

            // Emit transfer event
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }

        /// Transfer tokens from one account to another using allowance
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();

            // No-op if transferring to self or value is zero
            if from == to || value == 0 {
                return Ok(());
            }

            // Check allowance if caller is not the owner
            if caller != from {
                let current_allowance = self.allowance(from, caller);
                if current_allowance < value {
                    return Err(PSP22Error::InsufficientAllowance);
                }

                // Update allowance
                let new_allowance = current_allowance
                    .checked_sub(value)
                    .ok_or(PSP22Error::InsufficientAllowance)?;
                self.allowances.insert((from, caller), &new_allowance);

                // Emit approval event with new allowance
                self.env().emit_event(Approval {
                    owner: from,
                    spender: caller,
                    value: new_allowance,
                });
            }

            // Check balance
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            // Update balances
            let new_from_balance = from_balance
                .checked_sub(value)
                .ok_or(PSP22Error::InsufficientBalance)?;

            let to_balance = self.balance_of(to);
            let new_to_balance = to_balance
                .checked_add(value)
                .ok_or(PSP22Error::Custom(String::from("Overflow")))?;

            self.balances.insert(from, &new_from_balance);
            self.balances.insert(to, &new_to_balance);

            // Emit transfer event
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }

        /// Approve spender to spend tokens on behalf of caller
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let owner = self.env().caller();

            // No-op if approving self
            if owner == spender {
                return Ok(());
            }

            // Set allowance
            self.allowances.insert((owner, spender), &value);

            // Emit approval event
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        /// Increase the allowance granted to spender
        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();

            // No-op if increasing allowance for self or delta is zero
            if owner == spender || delta_value == 0 {
                return Ok(());
            }

            let current_allowance = self.allowance(owner, spender);
            let new_allowance = current_allowance
                .checked_add(delta_value)
                .ok_or(PSP22Error::Custom(String::from("Allowance overflow")))?;

            self.allowances.insert((owner, spender), &new_allowance);

            // Emit approval event
            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_allowance,
            });

            Ok(())
        }

        /// Decrease the allowance granted to spender
        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();

            // No-op if decreasing allowance for self or delta is zero
            if owner == spender || delta_value == 0 {
                return Ok(());
            }

            let current_allowance = self.allowance(owner, spender);
            if current_allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }

            let new_allowance = current_allowance
                .checked_sub(delta_value)
                .ok_or(PSP22Error::InsufficientAllowance)?;

            self.allowances.insert((owner, spender), &new_allowance);

            // Emit approval event
            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_allowance,
            });

            Ok(())
        }
    }

    impl PSP22Metadata for PspCoin {
        /// Returns the token name
        #[ink(message)]
        fn name(&self) -> Option<String> {
            Some(self.metadata.0.clone())
        }

        /// Returns the token symbol
        #[ink(message)]
        fn symbol(&self) -> Option<String> {
            Some(self.metadata.1.clone())
        }

        /// Returns the token decimals
        #[ink(message)]
        fn decimals(&self) -> u8 {
            self.metadata.2
        }
    }

    impl PSP22Mintable for PspCoin {
        /// Mint new tokens to caller's account
        #[ink(message)]
        fn mint(&mut self, value: u128) -> Result<(), PSP22Error> {
            let caller = self.env().caller();

            // No-op if value is zero
            if value == 0 {
                return Ok(());
            }

            // Update caller's balance
            let current_balance = self.balance_of(caller);
            let new_balance = current_balance
                .checked_add(value)
                .ok_or(PSP22Error::Custom(String::from("Balance overflow")))?;

            self.balances.insert(caller, &new_balance);

            // Update total supply
            self.total_supply = self.total_supply
                .checked_add(value)
                .ok_or(PSP22Error::Custom(String::from("Max supply exceeded")))?;

            // Emit transfer event with None as sender
            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value,
            });

            Ok(())
        }
    }

    impl PSP22Burnable for PspCoin {
        /// Burn tokens from caller's account
        #[ink(message)]
        fn burn(&mut self, value: u128) -> Result<(), PSP22Error> {
            let caller = self.env().caller();

            // No-op if value is zero
            if value == 0 {
                return Ok(());
            }

            // Check caller's balance
            let current_balance = self.balance_of(caller);
            if current_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            // Update caller's balance
            let new_balance = current_balance
                .checked_sub(value)
                .ok_or(PSP22Error::InsufficientBalance)?;

            self.balances.insert(caller, &new_balance);

            // Update total supply
            self.total_supply = self.total_supply
                .checked_sub(value)
                .ok_or(PSP22Error::InsufficientBalance)?;

            // Emit transfer event with None as recipient
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: None,
                value,
            });

            Ok(())
        }
    }
}
