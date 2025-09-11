#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::primitives::AccountId;

/// The IERC20 result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Emitted when `value` tokens are moved from one account (`from`) to another (`to`).
///
/// Note: `value` may be zero.
#[ink::event]
pub struct Transfer {
    #[ink(topic)]
    owner: AccountId,
    #[ink(topic)]
    spender: AccountId,
    value: u128,
}

/// Emitted when the allowance of a `spender` for an `owner` is set by a call to `approve`.
/// `value` is the new allowance.
#[ink::event]
pub struct Approval {
    #[ink(topic)]
    owner: AccountId,
    #[ink(topic)]
    spender: AccountId,
    value: u128,
}

/// A trait definition for an ERC-20 compatible token, following the IERC20 standard.
#[ink::trait_definition]
pub trait IERC20 {
    /// Returns the total token supply.
    #[ink(message)]
    fn total_supply(&self) -> u128;

    /// Returns the balance of the given `account`.
    #[ink(message)]
    fn balance_of(&self, account: AccountId) -> u128;

    /// Transfers `value` tokens from the caller's account to `to`.
    ///
    /// Returns `true` if the operation succeeded.
    ///
    /// Emits a `Transfer` event.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: u128) -> Result<bool>;

    /// Returns the remaining number of tokens that `spender` can spend
    /// on behalf of `owner` through `transfer_from`.
    #[ink(message)]
    fn allowance(&self, owner: AccountId, spender: AccountId) -> u128;

    /// Sets `value` as the allowance of `spender` over the caller’s tokens.
    ///
    /// Returns `true` if the operation succeeded.
    ///
    /// Emits an `Approval` event.
    #[ink(message)]
    fn approve(&mut self, spender: AccountId, value: u128) -> Result<bool>;

    /// Transfers `value` tokens from `from` to `to` using the allowance mechanism.
    /// `value` is then deducted from the caller’s allowance.
    ///
    /// Returns `true` if the operation succeeded.
    ///
    /// Emits a `Transfer` event.
    #[ink(message)]
    fn transfer_from(&mut self, from: AccountId, to: AccountId, value: u128) -> Result<bool>;
}

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Indicates an error related to the current balance of a sender.
    /// Used in transfers.
    InsufficientBalance {
        sender: AccountId,
        balance: u128,
        needed: u128,
    },

    /// Indicates a failure with the token sender. Used in transfers.
    InvalidSender { sender: AccountId },

    /// Indicates a failure with the token receiver. Used in transfers.
    InvalidReceiver { receiver: AccountId },

    /// Indicates a failure with the spender’s allowance. Used in transfers.
    InsufficientAllowance {
        spender: AccountId,
        allowance: u128,
        needed: u128,
    },

    /// Indicates a failure with the approver of a token to be approved.
    /// Used in approvals.
    InvalidApprover { approver: AccountId },

    /// Indicates a failure with the spender to be approved.
    /// Used in approvals.
    InvalidSpender { spender: AccountId },
}
