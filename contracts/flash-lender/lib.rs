#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod traits;

#[ink::contract]
mod flash_lender {
    use traits::IERC3156FlashLender::{Error, IERC3156FlashLender, Result};
    use ink::storage::Mapping;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct FlashLender {
        supported_tokens: Mapping<Address, bool>,
        fee: u128,
    }

    impl IERC3156FlashLender for FlashLender {
        /// See {traits.rs-flash_loan}
        #[ink(message)]
        fn flash_loan(
            &self,
            receiver: Address,
            token: Address,
            amount: u128,
            data: Vec<u8>,
        ) -> Result<bool> {
            self.supported_tokens
                .get(token)
                .unwrap_or(Error::UnsupportedCurrency)?;
            let fee = self._flash_fee(self.fee, amount);
            if !self._call_ERC20_transfer(receiver, token, amount) {
                return Err(Error::TransferFailed);
            }
            if !self._call_IERC3156FlashBorrower_callback(
                self.env.caller(),
                receiver,
                token,
                amount,
                fee,
                data,
            ) {
                return Err(Error::CallbackFailed);
            }
            if !self._call_ERC20_transfer_from(self.env().address(), receiver, token, amount) {
                return Err(Error::RepayFailed);
            }
            Ok(true)
        }

        /// See {traits.rs-max_flash_loan}
        #[ink(message)]
        fn flash_fee(&self, token: Address, amount: u128) -> Result<u128> {
            self.supported_tokens
                .get(token)
                .unwrap_or(Error::UnsupportedCurrency)?;
            self._flash_fee(self.fee, amount)
        }

        /// See {traits.rs-max_flash_loan}
        #[ink(message)]
        fn max_flash_loan(&self, token: Address) -> Result<u128> {
            let token_exists = self
                .supported_tokens
                .get(token)
                .unwrap_or(Error::UnsupportedCurrency)?;
            if token_exists {
                Ok(self._call_ERC20_balance_of(token, self.env().caller()))
            } else {
                Ok(0)
            }
        }
    }

    impl FlashLender {
        /// Creates a new [`FlashLender`].
        ///
        /// ## Params:
        /// - `supportedTokens`: Token contracts supported for flash lending.
        /// - `fee`: The percentage of the loan `amount` that needs to be repaid,
        ///   in addition to `amount`. (1 == 0.01%).
        #[ink(constructor)]
        pub fn new(supported_tokens: Vec<Address>, fee: u128) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                for token in supported_tokens {
                    contract.supported_tokens.insert(&token, ());
                }
                contract.fee = fee;
            })
        }

        /// Creates a default [`FlashLender`].
        #[ink(constructor)]
        pub fn default(supported_tokens: Vec<Address>, fee: u128) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.fee = 1;
            })
        }

        /// Internal function returning the fee to be charged for a given loan.  
        /// No safety checks are performed.
        ///
        /// ## Params:
        /// - `amount`: The amount of tokens lent.
        ///
        /// ## Returns:
        /// - `u256`: The fee to be charged on top of the returned principal.
        fn _flash_fee(fee: u128, amount: u128) -> u128 {
            amount * fee / 10000;
        }

        /// Calls the ERC20 `balance_of` function on a given token contract.
        ///
        /// ## Params:
        /// - `token`: Address of the ERC20 contract.
        /// - `account`: Address whose token balance should be queried.
        ///
        /// ## Returns:
        /// - The balance of `account` as `u128`.
        fn _call_ERC20_balance_of(token: Address, account: Address) -> u128 {
            build_call::<DefaultEnvironment>()
                .call(token)
                .call_v1()
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("balance_of")))
                        .push_arg(account),
                )
                .returns::<u128>()
                .invoke()
        }

        /// Calls the ERC20 `transfer` function on a given token contract.
        ///
        /// ## Params:
        /// - `receiver`: Address that will receive the tokens.
        /// - `token`: Address of the ERC20 contract.
        /// - `amount`: Amount of tokens to transfer.
        ///
        /// ## Returns:
        /// - A boolean indicating whether the transfer succeeded.
        fn _call_ERC20_transfer(receiver: Address, token: Address, amount: u128) -> Bool {
            build_call::<DefaultEnvironment>()
                .call(token)
                .call_v1()
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("transfer")))
                        .push_arg(receiver)
                        .push_arg(amount),
                )
                .returns::<bool>()
                .invoke()
        }

        /// Calls the ERC20 `transfer_from` function on a given token contract.
        ///
        /// The call attempts to transfer `amount + fee` tokens from the contract itself
        /// (`self.env().address()`) to the `receiver`.
        ///
        /// ## Params:
        /// - `this_address`: address of the smart contract executing this.
        /// - `receiver`: Address that will receive the tokens.
        /// - `token`: Address of the ERC20 contract.
        /// - `amount`: Principal amount to be transferred.
        /// - `fee`: Additional fee amount to be transferred.
        ///
        /// ## Returns:
        /// - A boolean indicating whether the transfer succeeded.
        fn _call_ERC20_transfer_from(
            this_address: Address,
            receiver: Address,
            token: Address,
            amount: u128,
            fee: u128,
        ) -> Bool {
            build_call::<DefaultEnvironment>()
                .call(token)
                .call_v1()
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("transfer_from")))
                        .push_arg(receiver)
                        .push_arg(this_address)
                        .push_arg(amount + fee),
                )
                .returns::<bool>()
                .invoke()
        }

        /// Calls the `on_flash_loan` callback on an `IERC3156FlashBorrower` contract.
        ///
        /// This is used by the flash lender to notify the borrower that it has received
        /// tokens and must execute its logic before repayment.
        ///
        /// ## Params:
        /// - `sender`: who initiated tx.
        /// - `receiver`: Address of the flash borrower contract.
        /// - `token`: Address of the ERC20 token contract used in the loan.
        /// - `amount`: Principal amount borrowed.
        /// - `fee`: Additional fee required for repayment.
        /// - `data`: Arbitrary bytes data passed through to the borrower.
        ///
        /// ## Returns:
        /// - A boolean indicating whether the callback succeeded.
        fn _call_IERC3156FlashBorrower_callback(
            sender: Address,
            receiver: Address,
            token: Address,
            amount: u128,
            fee: u128,
            data: Bytes,
        ) -> Bool {
            let sender = self.env.caller();
            build_call::<DefaultEnvironment>()
                .call(receiver)
                .call_v1()
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("on_flash_loan")))
                        .push_arg(sender)
                        .push_arg(token)
                        .push_arg(amount)
                        .push_arg(fee)
                        .push_arg(data),
                )
                .returns::<bool>()
                .invoke()
        }
    }
}
