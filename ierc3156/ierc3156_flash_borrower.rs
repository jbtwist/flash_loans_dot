//! Trait definition for a Flash Lender contract compatible with `IERC3156FlashLender`.
#![cfg_attr(not(feature = "std"), no_std, no_main)]

use crate::ierc3156_flash_lender::Error as LenderError;
use ierc20::Error as ERC20Error;
use ink::primitives::AccountId;

/// The Flash borrower result type.
pub type Result<T> = core::result::Result<T, Error>;

/// A trait for flash borrowing of ERC20 tokens, following the IERC3156 standard.
#[ink::trait_definition]
pub trait IERC3156FlashBorrower {
    /// ERC-3156 Flash loan callback.
    ///
    /// This function is called by the lender after the tokens have been
    /// transferred. It verifies the caller and initiator, decodes the action,
    /// and executes custom logic depending on the action type.
    ///
    /// ## Parameters:
    /// - `initiator`: The account that initiated the loan. Must be `self`.
    /// - `token`: The address of the token that was lent.
    /// - `amount`: The amount of tokens borrowed.
    /// - `fee`: The fee charged by the lender.
    /// - `data`: Encoded arbitrary data, usually used to signal the type of action.
    ///
    /// ## Returns:
    /// - A `bool` hash signaling successful execution of the callback.
    #[ink(message)]
    fn on_flash_loan(
        &self,
        initiator: AccountId,
        token: AccountId,
        amount: u128,
        fee: u128,
        data: Vec<u8>,
    ) -> Result<[u8; 32]>;

    /// Initiates a flash loan from the trusted lender.
    ///
    /// Prepares the encoded action data, checks and increases allowance if necessary,
    /// and requests a flash loan from the lender.
    ///
    /// ## Parameters:
    /// - `token`: The address of the token to borrow.
    /// - `amount`: The amount of tokens to borrow.
    #[ink(message)]
    fn flash_borrow(&self, token: AccountId, amount: u128) -> Result<()>;
}

/// The Flash Receiver error types.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Returned if the lender is not trusted.
    UntrustedLender,
    /// Returned if called by an untrusted loan initiator.
    UntrustedLoanInitiator,
    // Returned when decoding data failed.
    ScaleDecodingErr,
    /// Error related to ERC3156.
    ERC3156LenderError(LenderError),
    /// Error related to ERC20.
    ERC20Error(ERC20Error),
}
