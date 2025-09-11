//! Trait definition for a Flash Lender contract compatible with `IERC3156FlashLender`.
#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::primitives::AccountId;

/// The Flash lender result type.
pub type Result<T> = core::result::Result<T, Error>;

/// A trait for flash lending of ERC20 tokens, following the IERC3156 standard.
#[ink::trait_definition]
pub trait IERC3156FlashLender {
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
    ) -> Result<bool>;

    /// The fee to be charged for a given loan.
    ///
    /// ## Params:
    /// - `token`: The loan currency.
    /// - `amount`: The amount of tokens lent.
    ///
    /// ## Returns:
    /// - `u128`: The fee to be charged on top of the returned principal.
    #[ink(message)]
    fn flash_fee(&self, token: AccountId, amount: u128) -> Result<u128>;

    /// The amount of currency available to be lent.
    ///
    /// ## Params:
    /// - `token`: The loan currency.
    ///
    /// ## Returns:
    /// - `u128`: The amount of `token` that can be borrowed.
    #[ink(message)]
    fn max_flash_loan(&self, token: AccountId) -> Result<u128>;
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
    RepayFailed,
}
