#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flash_lender {
    use ink::{
        env::{
            call::{build_call, ExecutionInput, Selector},
            hash::Keccak256,
            DefaultEnvironment,
        },
        storage::Mapping,
    };
    use IERC3156::ierc3156_flash_lender::{Error, IERC3156FlashLender, Result};

    #[ink(storage)]
    pub struct FlashLender {
        supported_tokens: Mapping<AccountId, bool>,
        fee: u128, // 1 = 0.01%
    }

    impl IERC3156FlashLender for FlashLender {
        /// Loan `amount` tokens to `receiver`, and take them back plus a `flashFee` after the callback.
        ///
        /// ## Params:
        /// - `receiver`: The contract receiving the tokens.  
        ///   Must implement the `onFlashLoan(address user, uint256 amount, uint256 fee, bytes calldata)` interface.
        /// - `token`: The loan currency.
        /// - `amount`: The amount of tokens lent.
        /// - `data`: A data parameter to be passed on to the `receiver` for any custom use.
        ///
        /// ## Returns:
        /// - `bool`: True if the flash loan succeeds.
        #[ink(message)]
        fn flash_loan(
            &self,
            receiver: AccountId,
            token: AccountId,
            amount: u128,
            data: Vec<u8>,
        ) -> Result<bool> {
            self.supported_tokens
                .get(token)
                .ok_or(Error::UnsupportedCurrency)?;
            let fee = self._flash_fee(self.fee, amount);
            if !self._call_erc20_transfer(receiver, token, amount) {
                return Err(Error::TransferFailed);
            }
            if self._call_ierc3156_flash_borrower_callback(
                self.env().caller(),
                receiver,
                token,
                amount,
                fee,
                data,
            ) != self
                .env()
                .hash_bytes::<Keccak256>(b"ERC3156FlashBorrower.onFlashLoan")
            {
                return Err(Error::CallbackFailed);
            }
            if !self._call_erc20_transfer_from(
                self.env().account_id(),
                receiver,
                token,
                amount,
                fee,
            ) {
                return Err(Error::RepayFailed);
            }
            Ok(true)
        }

        /// The fee to be charged for a given loan.
        ///
        /// ## Params:
        /// - `token`: The loan currency.
        /// - `amount`: The amount of tokens lent.
        ///
        /// ## Returns:
        /// - `u128`: The fee to be charged on top of the returned principal.
        #[ink(message)]
        fn flash_fee(&self, token: AccountId, amount: u128) -> Result<u128> {
            self.supported_tokens
                .get(token)
                .ok_or(Error::UnsupportedCurrency)?;
            Ok(self._flash_fee(self.fee, amount))
        }

        /// The amount of currency available to be lent.
        ///
        /// ## Params:
        /// - `token`: The loan currency.
        ///
        /// ## Returns:
        /// - `u128`: The amount of `token` that can be borrowed.
        #[ink(message)]
        fn max_flash_loan(&self, token: AccountId) -> Result<u128> {
            let token_exists = self
                .supported_tokens
                .get(token)
                .ok_or(Error::UnsupportedCurrency)?;
            if token_exists {
                Ok(self._call_erc20_balance_of(token, self.env().caller()))
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
        pub fn new(_supported_tokens: Vec<AccountId>, fee: u128) -> Self {
            let mut supported_tokens = Mapping::default();
            for token in _supported_tokens {
                supported_tokens.insert(&token, &true);
            }
            Self {
                supported_tokens,
                fee,
            }
        }

        /// Internal function returning the fee to be charged for a given loan.  
        /// No safety checks are performed.
        ///
        /// ## Params:
        /// - `amount`: The amount of tokens lent.
        ///
        /// ## Returns:
        /// - `u256`: The fee to be charged on top of the returned principal.
        fn _flash_fee(&self, fee: u128, amount: u128) -> u128 {
            amount * fee / 10000
        }

        /// Calls the ERC20 `balance_of` function on a given token contract.
        ///
        /// ## Params:
        /// - `token`: AccountId of the ERC20 contract.
        /// - `account`: AccountId whose token balance should be queried.
        ///
        /// ## Returns:
        /// - The balance of `account` as `u128`.
        fn _call_erc20_balance_of(&self, token: AccountId, account: AccountId) -> u128 {
            build_call::<DefaultEnvironment>()
                .call(token)
                .call_v1()
                .gas_limit(1000)
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
        /// - `receiver`: AccountId that will receive the tokens.
        /// - `token`: AccountId of the ERC20 contract.
        /// - `amount`: Amount of tokens to transfer.
        ///
        /// ## Returns:
        /// - A boolean indicating whether the transfer succeeded.
        fn _call_erc20_transfer(
            &self,
            receiver: AccountId,
            token: AccountId,
            amount: u128,
        ) -> bool {
            build_call::<DefaultEnvironment>()
                .call(token)
                .call_v1()
                .gas_limit(1000)
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
        /// - `receiver`: AccountId that will receive the tokens.
        /// - `token`: AccountId of the ERC20 contract.
        /// - `amount`: Principal amount to be transferred.
        /// - `fee`: Additional fee amount to be transferred.
        ///
        /// ## Returns:
        /// - A boolean indicating whether the transfer succeeded.
        fn _call_erc20_transfer_from(
            &self,
            this_address: AccountId,
            receiver: AccountId,
            token: AccountId,
            amount: u128,
            fee: u128,
        ) -> bool {
            build_call::<DefaultEnvironment>()
                .call(token)
                .call_v1()
                .gas_limit(1000)
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
        /// - `receiver`: AccountId of the flash borrower contract.
        /// - `token`: AccountId of the ERC20 token contract used in the loan.
        /// - `amount`: Principal amount borrowed.
        /// - `fee`: Additional fee required for repayment.
        /// - `data`: Arbitrary bytes data passed through to the borrower.
        ///
        /// ## Returns:
        /// - A boolean indicating whether the callback succeeded.
        fn _call_ierc3156_flash_borrower_callback(
            &self,
            sender: AccountId,
            receiver: AccountId,
            token: AccountId,
            amount: u128,
            fee: u128,
            data: Vec<u8>,
        ) -> [u8; 32] {
            build_call::<DefaultEnvironment>()
                .call(receiver)
                .call_v1()
                .gas_limit(1000)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("on_flash_loan")))
                        .push_arg(sender)
                        .push_arg(token)
                        .push_arg(amount)
                        .push_arg(fee)
                        .push_arg(data),
                )
                .returns::<[u8; 32]>()
                .invoke()
        }
    }
}
