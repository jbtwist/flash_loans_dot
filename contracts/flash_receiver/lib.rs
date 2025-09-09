#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod Receiver {

    pub enum Action {
        Arbitrage,
        Other,
    }

    #[ink(storage)]
    pub struct Receiver {
        /// Stores the receiver lender's AccountId.
        lender: AccountId,
        /// Stores the last action performed.
        action: Action,
    }

    impl Receiver {
        /// Constructor that initializes the `bool` value to the given `init_value`.
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
            let decoded_action = self
                .decode_action(data)
                .map_err(|_| "FlashBorrower: Unable to decode action".to_string())?;

            match decoded_action {
                Action::Arbitrage => {
                    // Mock an arbitrage action, this should be an EV+ operation
                    self.action = Action::Arbitrage;
                    // Profitable logic would go here
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
                _ => return false,
            }

            true
        }

        /// Decodes the data into an action
        #[ink(message)]
        pub fn decode_action(&self, data: Vec<u8>) -> Result<Action, scale::Error> {
            Action::decode(&mut &data[..])
        }
    }
}
