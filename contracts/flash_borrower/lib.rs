#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flash_receiver {
    use ink::env::hash::Keccak256;
    use ink::prelude::vec::Vec;
    use ink::scale::{Decode, Encode, Error as ScaleError};
    use IERC20::IERC20;
    use IERC3156::ierc3156_flash_borrower::{Error, IERC3156FlashBorrower, Result};
    use IERC3156::ierc3156_flash_lender::IERC3156FlashLender;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Action {
        Normal,
        Other,
    }

    #[ink(storage)]
    pub struct FlashBorrower {
        /// Stores the receiver lender's AccountId.
        lender: AccountId,
        /// Stores the last action performed.
        action: Action,
    }

    impl IERC3156FlashBorrower for FlashBorrower {
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
            amount: Balance,
            fee: Balance,
            data: Vec<u8>,
        ) -> Result<[u8; 32]> {
            let caller = self.env().caller();
            if caller != self.lender {
                return Err(Error::UntrustedLender);
            }
            if initiator != self.env().account_id() {
                return Err(Error::UntrustedLoanInitiator);
            }

            let decoded_action = self.decode_action(data)?;

            match decoded_action {
                Action::Normal => {
                    // Mock an arbitrage action, this should be an EV+ operation
                    // TODO: Profitable logic would go here
                    // Emit event
                }
                Action::Other => {
                    // Perform other action
                }
            }
            Ok(self
                .env()
                .hash_bytes::<Keccak256>(b"ERC3156FlashBorrower.onFlashLoan"))
        }

        /// Initiates a flash loan from the trusted lender.
        ///
        /// Prepares the encoded action data, checks and increases allowance if necessary,
        /// and requests a flash loan from the lender.
        ///
        /// ## Parameters:
        /// - `token`: The address of the token to borrow.
        /// - `amount`: The amount of tokens to borrow.
        #[ink(message)]
        fn flash_borrow(&self, token: AccountId, amount: u128) -> Result<()> {
            let mut erc20: ink::contract_ref!(IERC20) = token.into();
            let lender: ink::contract_ref!(IERC3156FlashLender) = self.lender.into();
            let allowance = erc20.allowance(self.env().account_id(), self.lender);
            let fee = lender
                .flash_fee(token, amount)
                .map_err(|_| Error::UnsupportedCurrency)?;
            let repayment = amount + fee;
            erc20.approve(self.lender, allowance + repayment);
            lender
                .flash_loan(
                    self.env().account_id(),
                    token,
                    amount,
                    self.encode_action(Action::Normal),
                )
                .map_err(|_| Error::LoanFailed)?;
            Ok(())
        }
    }

    impl FlashBorrower {
        /// Creates a new [`FlashBorrower`] instance.
        ///
        /// ## Parameters:
        /// - `lender_`: The trusted flash lender contract.
        #[ink(constructor)]
        pub fn new(lender: AccountId) -> Self {
            Self {
                lender,
                action: Action::Normal,
            }
        }

        /// Decodes the data into an action
        fn decode_action(&self, data: Vec<u8>) -> Result<Action> {
            Action::decode(&mut &data[..]).map_err(|_| Error::ScaleDecodingErr)
        }

        /// Encodes action into data
        fn encode_action(&self, action: Action) -> Vec<u8> {
            Action::encode(&action)
        }
    }
}
