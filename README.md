# flash_loans_dot
We're implementing flash loans (EIP-3156) solution into the Polkadot echosystem

---

In order to develop using ink, we have to follow the official tutorial here: https://use.ink/docs/v5/getting-started/setup/

Step 1: Install cargo-contract: cargo install cargo-contract

Step 2: Install substrate-contracts-node: cargo install contracts-node

Step 3: Use ink CLI to generate an initial smart contract with specialized #[ink(…)] attribute macros.

Step 4: Make sure you're in your working directory and execute: cargo contract new flipper