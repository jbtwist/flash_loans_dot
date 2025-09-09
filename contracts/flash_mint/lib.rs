#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flash_mint_contract {
    #[ink(storage)]
    pub struct Mint {
        pub fee: u128,
        pub callback_success: bool,
    }

    impl Mint {
        #[ink(constructor)]
        pub fn new(_fee: u128) -> Self {
            Self {
                fee: _fee
            }
        }

        #[ink(message)]
        fn max_flash_loan(token: AccountId) -> u128 {
            // TODO totalSupply come from the ERC20 token standard
            // return type(uint256).max - totalSupply();
        }

        #[ink(message)]
        fn flash_fee(
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
            token: AddressId,
            amount: u128
        ) -> u128 {
            return amount * fee / 10000;
        }

        #[ink(message)]
        fn flash_loan(
            receiver: AccountId,
            token: AccountId,
            amount: u128,
            data: Vec<u8>,
        ) -> bool {
            ensure!(
                token == Self::env::address(),
                "FlashMinter: Unsupported currency"
            );

            let fee: u128 = Self::_flash_fee(token, amount);
            // TODO mint come from the ERC20 token standard
            // _mint(receiver, amount);
            ensure!(
                receiver.onFlashLoan(Self::env::caller(), token, amount, fee, data) == callback_success,
                "FlashMinter: Callback failed"
            );
            // TODO allowance come from the ERC20 token standard
            let _allowance: u128 = allowance(receiver, Self::env::address());
            ensure!(
                _allowance >= (amount + fee),
                "FlashMinter: Repay not approved"
            );
            // TODO _approve and _burn come from the ERC20 token standard
            _approve(receiver, Self::env::address(), _allowance - (amount + fee));
            _burn(receiver, amount + fee);
            return true;
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