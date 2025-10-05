#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;

#[ink::contract]
mod psp_coin {
    use ink::{storage::Mapping, prelude::vec::Vec, prelude::string::String};

    use crate::data::PSP22Error;

    /// Storage structure for the PSP-22 token
    #[ink(storage)]
    pub struct PspCoin {
        /// Total supply of tokens
        total_supply: u128,
        /// Mapping from account to token balance
        balances: Mapping<Address, u128>,
        /// Nested mapping for allowances (owner, spender) -> amount
        allowances: Mapping<(Address, Address), u128>,
        /// Token metadata (name, symbol, decimals)
        metadata: (String, String, u8),
    }

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Option<Address>,
        #[ink(topic)]
        pub to: Option<Address>,
        pub value: u128,
    }

    /// Event emitted when an approval is granted
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        pub owner: Address,
        #[ink(topic)]
        pub spender: Address,
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

    impl PspCoin {
        /// Returns the total token supply
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        /// Returns the balance of the specified owner
        #[ink(message)]
        pub fn balance_of(&self, owner: Address) -> u128 {
            self.balances.get(owner).unwrap_or(0)
        }

        /// Returns the allowance granted by owner to spender
        #[ink(message)]
        pub fn allowance(&self, owner: Address, spender: Address) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        /// Transfer tokens from caller to recipient
        #[ink(message)]
        pub fn transfer(&mut self, to: Address, value: u128, _data: Vec<u8>) -> Result<(), PSP22Error> {
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
        pub fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
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
        pub fn approve(&mut self, spender: Address, value: u128) -> Result<(), PSP22Error> {
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
        pub fn increase_allowance(
            &mut self,
            spender: Address,
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
        pub fn decrease_allowance(
            &mut self,
            spender: Address,
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

        /// Returns the token name
        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            Some(self.metadata.0.clone())
        }

        /// Returns the token symbol
        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            Some(self.metadata.1.clone())
        }

        /// Returns the token decimals
        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.metadata.2
        }

        /// Mint new tokens to caller's account
        #[ink(message)]
        pub fn mint(&mut self, value: u128) -> Result<(), PSP22Error> {
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

        /// Burn tokens from caller's account
        #[ink(message)]
        pub fn burn(&mut self, value: u128) -> Result<(), PSP22Error> {
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let contract = PspCoin::new();
            assert_eq!(contract.total_supply(), 0);
        }

        #[ink::test]
        fn new_with_supply_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let contract = PspCoin::new_with_supply(1000);
            assert_eq!(contract.total_supply(), 1000);
            assert_eq!(contract.balance_of(accounts.alice), 1000);
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            // Transfer from Alice to Bob
            assert_eq!(contract.transfer(accounts.bob, 100, vec![]), Ok(()));
            assert_eq!(contract.balance_of(accounts.alice), 900);
            assert_eq!(contract.balance_of(accounts.bob), 100);
        }

        #[ink::test]
        fn transfer_fails_insufficient_balance() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(100);

            // Try to transfer more than balance
            assert_eq!(
                contract.transfer(accounts.bob, 200, vec![]),
                Err(PSP22Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn transfer_to_self_is_noop() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            // Transfer to self should be no-op
            assert_eq!(contract.transfer(accounts.alice, 100, vec![]), Ok(()));
            assert_eq!(contract.balance_of(accounts.alice), 1000);
        }

        #[ink::test]
        fn approve_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.approve(accounts.bob, 200), Ok(()));
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), 200);
        }

        #[ink::test]
        fn transfer_from_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            // Alice approves Bob to spend 200 tokens
            assert_eq!(contract.approve(accounts.bob, 200), Ok(()));

            // Switch to Bob's account
            ink::env::test::set_caller(accounts.bob);

            // Bob transfers 100 tokens from Alice to Charlie
            assert_eq!(
                contract.transfer_from(accounts.alice, accounts.charlie, 100, vec![]),
                Ok(())
            );

            assert_eq!(contract.balance_of(accounts.alice), 900);
            assert_eq!(contract.balance_of(accounts.charlie), 100);
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), 100);
        }

        #[ink::test]
        fn transfer_from_fails_insufficient_allowance() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            // Alice approves Bob to spend 50 tokens
            assert_eq!(contract.approve(accounts.bob, 50), Ok(()));

            // Switch to Bob's account
            ink::env::test::set_caller(accounts.bob);

            // Bob tries to transfer 100 tokens (more than allowance)
            assert_eq!(
                contract.transfer_from(accounts.alice, accounts.charlie, 100, vec![]),
                Err(PSP22Error::InsufficientAllowance)
            );
        }

        #[ink::test]
        fn increase_allowance_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.approve(accounts.bob, 100), Ok(()));
            assert_eq!(contract.increase_allowance(accounts.bob, 50), Ok(()));
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), 150);
        }

        #[ink::test]
        fn decrease_allowance_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.approve(accounts.bob, 100), Ok(()));
            assert_eq!(contract.decrease_allowance(accounts.bob, 30), Ok(()));
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), 70);
        }

        #[ink::test]
        fn decrease_allowance_fails_insufficient() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.approve(accounts.bob, 50), Ok(()));

            // Try to decrease more than current allowance
            assert_eq!(
                contract.decrease_allowance(accounts.bob, 100),
                Err(PSP22Error::InsufficientAllowance)
            );
        }

        #[ink::test]
        fn mint_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new();

            assert_eq!(contract.mint(500), Ok(()));
            assert_eq!(contract.total_supply(), 500);
            assert_eq!(contract.balance_of(accounts.alice), 500);
        }

        #[ink::test]
        fn burn_works() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.burn(300), Ok(()));
            assert_eq!(contract.total_supply(), 700);
            assert_eq!(contract.balance_of(accounts.alice), 700);
        }

        #[ink::test]
        fn burn_fails_insufficient_balance() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(100);

            // Try to burn more than balance
            assert_eq!(
                contract.burn(200),
                Err(PSP22Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn metadata_works() {
            let contract = PspCoin::new();

            assert_eq!(contract.name(), Some(String::from("PSP Coin")));
            assert_eq!(contract.symbol(), Some(String::from("PSP")));
            assert_eq!(contract.decimals(), 18);
        }

        #[ink::test]
        fn zero_value_transfer_is_noop() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.transfer(accounts.bob, 0, vec![]), Ok(()));
            assert_eq!(contract.balance_of(accounts.alice), 1000);
            assert_eq!(contract.balance_of(accounts.bob), 0);
        }

        #[ink::test]
        fn zero_value_mint_is_noop() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new();

            assert_eq!(contract.mint(0), Ok(()));
            assert_eq!(contract.total_supply(), 0);
        }

        #[ink::test]
        fn zero_value_burn_is_noop() {
            let accounts = ink::env::test::default_accounts();
            ink::env::test::set_caller(accounts.alice);

            let mut contract = PspCoin::new_with_supply(1000);

            assert_eq!(contract.burn(0), Ok(()));
            assert_eq!(contract.total_supply(), 1000);
        }
    }
}
