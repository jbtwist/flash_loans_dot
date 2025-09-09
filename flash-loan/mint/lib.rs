#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flash_mint_contract {
    #[ink(storage)]
    pub struct Mint {}

    impl Mint {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message, payable)]
        pub fn mint(&self, to: AccountId, amount: Balance) {}

        #[ink(message)]
        pub fn burn(&self, from: AccountId, amount: Balance) {}
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