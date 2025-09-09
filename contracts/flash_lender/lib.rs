#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod traits;

#[ink::contract]
mod flash_lender {
	use traits::FlashLender;

	#[ink(storage)]
	#[derive(SpreadAllocate)]
	pub struct FlashLender {
		supported_tokens: ink_storage::Mapping<AccountId, bool>,
		fee : u128
	}

	/// The Flash Lender error types.
	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		/// Returned if currency is not available.
		UnsupportedCurrency,
		/// Returned if external `IERC20` transfer call failed.
		TransferFailed,
		/// Returned if external `IERC3156FlashBorrower` callback failed.
		CallbackFailed,
		/// Returned if external `IERC20` repay call failed.
		RepayFailed
	}

	/// The Flash lender result type.
	pub type Result<T> = core::result::Result<T, Error>;


	impl FlashLender {
		/// Creates a new [`FlashLender`].
		///
		/// ## Params:
		/// - `supportedTokens`: Token contracts supported for flash lending.
		/// - `fee`: The percentage of the loan `amount` that needs to be repaid,
		///   in addition to `amount`. (1 == 0.01%).
		#[ink(constructor)]
		pub fn new(
			supported_tokens: Vec<AccountId>,
			fee: u128
		) -> Self {
			ink_lang::utils::initialize_contract(|contract: &mut Self| {
				for token in supported_tokens {
					contract.supported_tokens.insert(&token, ());
				}
				contract.fee = fee;
			})
		}

		/// Creates a default [`FlashLender`].
		#[ink(constructor)]
		pub fn default(
			supported_tokens: Vec<AccountId>,
			fee: u128
		) -> Self {
			ink_lang::utils::initialize_contract(|contract: &mut Self| {
				contract.fee = 1;
			})
		}

		/// Internal function returning the fee to be charged for a given loan.  
		/// No safety checks are performed.
		///
		/// ## Params:
		/// - `token`: The loan currency.
		/// - `amount`: The amount of tokens lent.
		///
		/// ## Returns:
		/// - `u256`: The fee to be charged on top of the returned principal.
		fn _flash_fee(
			token: Address,
			amount: u128
		) -> u128 {
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
		fn _call_ERC20_balance_of(
			token: Address,
			account: Address,
		) -> u128 {
			build_call::<DefaultEnvironment>()
				.call(token)
				.call_v1()
				.gas_limit(1000)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("balance_of")))
						.push_arg(account)
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
		fn _call_ERC20_transfer(
			receiver: Address,
			token: Address,
			amount: u128
		) -> Bool {
			build_call::<DefaultEnvironment>()
				.call(token)
				.call_v1()
				.gas_limit(1000)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("transfer")))
						.push_arg(receiver)
						.push_arg(amount)
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
		/// - `receiver`: Address that will receive the tokens.
		/// - `token`: Address of the ERC20 contract.
		/// - `amount`: Principal amount to be transferred.
		/// - `fee`: Additional fee amount to be transferred.
		///
		/// ## Returns:
		/// - A boolean indicating whether the transfer succeeded.
		fn _call_ERC20_transfer_from(
			receiver: Address,
			token: Address,
			amount: u128,
			fee: u128
		) -> Bool {
			let this_address : Address = self.env().address();
			build_call::<DefaultEnvironment>()
				.call(token)
				.call_v1()
				.gas_limit(1000)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("transfer_from")))
						.push_arg(receiver)
						.push_arg(this_address)
						.push_arg(amount + fee)
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
		/// - `receiver`: Address of the flash borrower contract.
		/// - `token`: Address of the ERC20 token contract used in the loan.
		/// - `amount`: Principal amount borrowed.
		/// - `fee`: Additional fee required for repayment.
		/// - `data`: Arbitrary bytes data passed through to the borrower.
		///
		/// ## Returns:
		/// - A boolean indicating whether the callback succeeded.
		fn _call_IERC3156FlashBorrower_callback(
			receiver: Address,
			token: Address,
			amount: u128,
			fee: u128,
			data: Bytes
		) -> Bool {
			let sender = self.env.caller();
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
						.push_arg(data)
				)
				.returns::<bool>()
				.invoke()
		}
	}


	impl FlashLender for FlashLender {
		/// Loan `amount` tokens to `receiver`, and takes it back plus a `flashFee` after the callback.
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
		pub fn flash_loan(
			receiver: Address,
			token: Address,
			amount: u128,
			data: Bytes
		) -> Result<Bool> {
			self.supported_tokens.get(token).unwrap_or(Error::UnsupportedCurrency)?;
			let fee = Self::_flash_fee(token, amount);call_IERC3156FlashBorrower_callback
			assert!(Self::_call_ERC20_transfer(receiver, token, amount), Error::TransferFailed);
			assert!(Self::_call_IERC3156FlashBorrower_callback(receiver, token, amount, fee, data), Error::CallbackFailed);
			assert!(Self::_call_ERC20_transfer_from(receiver, token, amount), Error::RepayFailed);
			Ok(true)
        }

		/// The fee to be charged for a given loan.
		///
		/// ## Params:
		/// - `token`: The loan currency.
		/// - `amount`: The amount of tokens lent.
		///
		/// ## Returns:
		/// - `u256`: The fee to be charged on top of the returned principal.
		#[ink(message)]
		pub fn flash_fee(
			token: Address,
			amount: u128
		) -> Result<u128> {
			self.supported_tokens.get(token).unwrap_or(Error::UnsupportedCurrency)?;
			Self::_flash_fee(token, amount)
		}

		/// The amount of currency available to be lent.
		///
		/// ## Params:
		/// - `token`: The loan currency.
		///
		/// ## Returns:
		/// - `u256`: The amount of `token` that can be borrowed.
		#[ink(message)]
		pub fn max_flash_loan(
			token: Address
		) -> Result<u128> {
			let token_exists = self.supported_tokens.get(token).unwrap_or(Error::UnsupportedCurrency)?;
			if token_exists {
				Ok(Self::_call_ERC20_balance_of(token, self.env().caller()))
			} else {
				Ok(0)
			}
		}
	}
}