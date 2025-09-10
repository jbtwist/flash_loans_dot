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
    fn allowance(owner: Address, spender: Address) -> u128 {}
    #[ink::trait_definition]
    fn _approve(owner: Address, spender: Address, amount: u128) {}
    #[ink::trait_definition]
    fn _burn(account: Address, amount: u128) {}
    #[ink::trait_definition]
    fn _mint(account: Address, amount: u128) {}

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {}

    pub type Result<T> = core::result::Result<T, Error>;

    impl Mint {
        pub fn new(_fee: u128) -> Self {
            Self { fee: _fee }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(5) // default fee is 0.05%
        }

        #[ink(message)]
        pub fn max_flash_loan(&self, token: Address) -> Result<u128> {
            return U128.MAX - self.total_supply();
        }

        #[ink(message)]
        pub fn flash_fee(&self, token: Address, amount: u128) -> u128 {
            assert!(
                token == ink::env::address(),
                "FlashMinter: Unsupported currency"
            );
            self._flash_fee(token, amount)
        }

        fn _flash_fee(fee: u128, amount: u128) -> u128 {
            amount * fee / 10000
        }

        #[ink(message)]
        fn flash_loan(
            &self,
            receiver: Address,
            token: Address,
            amount: u128,
            data: Vec<u8>,
        ) -> Result<bool> {
            assert!(
                token == ink::env::address(),
                "FlashMinter: Unsupported currency"
            );

            let fee: u128 = self.flash_fee(token, amount);
            let sender = self.env.caller();

            build_call::<DefaultEnvironment>()
                .call(receiver)
                .call_v1()
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("mint")))
                        .push_arg(receiver)
                        .push_arg(amount),
                )
                .returns::<bool>()
                .invoke();

            assert!(
                receiver.onFlashLoan(ink::env::caller(), token, amount, fee, data)
                    == callback_success,
                "FlashMinter: Callback failed"
            );
            let _allowance: u128 = Self::allowance(receiver, ink::env::address());
            assert!(
                _allowance >= (amount + fee),
                "FlashMinter: Repay not approved"
            );
            self::_approve(receiver, ink::env::address(), _allowance - (amount + fee));
            self::_burn(receiver, amount + fee);
            Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn mint_happy_path_testing() {
        // let flash_mint_contract =
        assert_ok!(FlashMinter::default());
    }

    #[ink::test]
    fn mint_errors_testing() {}
}
