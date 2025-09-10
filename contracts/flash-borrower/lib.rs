#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod traits;

#[ink::contract]
mod flash_receiver {
    use ink::prelude::vec::Vec;
    use ink::scale::{Decode, Error as ScaleError};
    use crate::traits::{IERC3156FlashBorrower, Action};

    #[ink(storage)]
    pub struct FlashBorrower {
        /// Stores the receiver lender's AccountId.
        lender: AccountId,
        /// Stores the last action performed.
        action: Action,
    }

    impl IERC3156FlashBorrower for FlashBorrower {
        /// See {traits.rs-on_flash_loan}
        #[ink(message)]
        fn on_flash_loan(
            &mut self,
            initiator: AccountId,
            amount: Balance,
            fee: Balance,
            data: Vec<u8>,
        ) -> bool {
            let caller = self.env().caller();
            if caller != self.lender {
                return false;
            }
            if initiator != self.env().account_id() {
                return false;
            }

            let decoded_action = match self.decode_action(data) {
                Ok(action) => action,
                Err(_) => return false,
            };

            match decoded_action {
                Action::Arbitrage => {
                    // Mock an arbitrage action, this should be an EV+ operation
                    // TODO: Profitable logic would go here
                    // Emit event
                }
                Action::Other => {
                    // Perform other action
                }
            }
            true
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
                action: Action::Arbitrage,
            }
        }

        /// Decodes the data into an action
        fn decode_action(&self, data: Vec<u8>) -> Result<Action, ScaleError> {
            Action::decode(&mut &data[..])
        }
    }
}
