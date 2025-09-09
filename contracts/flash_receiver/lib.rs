#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod Receiver {
    use ink::prelude::vec::Vec;
    use ink::scale::{Decode, Error as ScaleError};

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Action {
        Arbitrage,
        Other,
    }

    #[ink(event)]
    pub struct ActionPerformed {
        #[ink(topic)]
        action: Action,
        amount: Balance,
        fee: Balance,
    }

    #[ink(storage)]
    pub struct Receiver {
        /// Stores the receiver lender's AccountId.
        lender: AccountId,
        /// Stores the last action performed.
        action: Action,
    }

    impl Receiver {
        /// Constructor that initializes the receiver with a lender.
        #[ink(constructor)]
        pub fn new(lender: AccountId) -> Self {
            Self {
                lender,
                action: Action::Arbitrage,
            }
        }

        /// Implements the logic for handling a flash loan.
        #[ink(message)]
        pub fn on_flash_loan(
            &mut self,
            initiator: AccountId,
            amount: Balance,
            fee: Balance,
            data: Vec<u8>,
        ) -> bool {
            let caller = Self::env().caller();
            if caller != self.lender {
                return false;
            }
            if initiator != Self::env().account_id() {
                return false;
            }

            let decoded_action = match self.decode_action(data) {
                Ok(action) => action,
                Err(_) => return false,
            };

            match decoded_action {
                Action::Arbitrage => {
                    // Mock an arbitrage action, this should be an EV+ operation
                    self.action = Action::Arbitrage;
                    // TODO: Profitable logic would go here
                    // Emit event
                    Self::env().emit_event(ActionPerformed {
                        action: Action::Arbitrage,
                        amount,
                        fee,
                    });
                }
                Action::Other => {
                    // Perform other action
                    self.action = Action::Other;
                    Self::env().emit_event(ActionPerformed {
                        action: Action::Other,
                        amount,
                        fee,
                    });
                }
            }

            true
        }

        /// Decodes the data into an action
        fn decode_action(&self, data: Vec<u8>) -> Result<Action, ScaleError> {
            Action::decode(&mut &data[..])
        }
    }
}
