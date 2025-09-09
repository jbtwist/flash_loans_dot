#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flash_mint_contract {
    #[ink(storage)]
    pub struct Mint {
        pub fee: u128,
        pub callback_success: bool,
    }

    #[ink::trait_definition]
    fn total_supply() -> u128 {}
    #[ink::trait_definition]
    fn allowance(owner: AccountId, spender: AccountId) -> u128 {}
    #[ink::trait_definition]
    fn _approve(owner: AccountId, spender: AccountId, amount: u128) {}
    #[ink::trait_definition]
    fn _burn(account: AccountId, amount: u128) {}
    #[ink::trait_definition]
    fn _mint(account: AccountId, amount: u128) {}

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {

    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Mint {
        pub fn new(_fee: u128) -> Self {
            Self {
                fee: _fee
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(5) // default fee is 0.05%
        }

        #[ink(message)]
        fn max_flash_loan(&self, token: AccountId) -> Result<u128> {
            return U128.MAX - Self::total_supply();
        }

        #[ink(message)]
        fn flash_fee(
            &self,
            token: AccountId,
            amount: u128,
        ) -> u128 {
            ensure!(
                token == self::env::address(),
                "FlashMinter: Unsupported currency"
            );
            return Self::_flash_fee(token, amount);
        }

        fn _flash_fee(
            &self,
            token: AddressId,
            amount: u128
        ) -> Result<u128> {
            return amount * fee / 10000;
        }

        #[ink(message)]
        fn flash_loan(
            &self,
            receiver: AccountId,
            token: AccountId,
            amount: u128,
            data: Vec<u8>,
        ) -> Result<bool> {
            ensure!(
                token == Self::env::address(),
                "FlashMinter: Unsupported currency"
            );

            let fee: u128 = Self::_flash_fee(token, amount);
            Self::_mint(receiver, amount);
            ensure!(
                receiver.onFlashLoan(Self::env::caller(), token, amount, fee, data) == callback_success,
                "FlashMinter: Callback failed"
            );
            let _allowance: u128 = Self::allowance(receiver, Self::env::address());
            ensure!(
                _allowance >= (amount + fee),
                "FlashMinter: Repay not approved"
            );
            self::_approve(receiver, self::env::address(), _allowance - (amount + fee));
            self::_burn(receiver, amount + fee);
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_happy_path_testing() {}

    #[test]
    fn mint_errors_testing() {}
}