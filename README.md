# Solana Constant Product AMM 

A highly optimized, decentralized Automated Market Maker (AMM) smart contract built on Solana. This protocol implements the core constant product invariant to facilitate permissionless token swapping, dynamic pricing, and decentralized liquidity provisioning. 

Built with Rust and the Anchor framework, this repository serves as a robust backend engine for decentralized exchanges (DEXs).

## 🧮 Mathematical Invariants

The AMM is governed by two core mathematical principles:
* **Pricing (Constant Product):** Trades are executed along the $x \cdot y = k$ bonding curve. The protocol ensures that the product of the reserve balances remains constant (or increases via fees) after every swap.
* **Liquidity Minting (Geometric Mean):** Initial liquidity pool (LP) shares are minted based on the geometric mean of the deposited assets: $L = \sqrt{x \cdot y}$.

## ⚙ Core Instructions

1. **`initialize`**: Provisions a new central `AmmPool` state account. Initializes the vault token accounts and the LP token mint for a specific trading pair.
2. **`deposit`**: Allows users to supply dual-sided liquidity into the pool's vaults. Mints proportional LP tokens to the depositor's wallet representing their share of the pool.
3. **`swap`**: Executes a trade between Token A and Token B. Automatically calculates the output amount based on the constant product curve, deducts protocol fees, and transfers assets between the user and the pool vaults.
4. **`withdraw`**: Allows Liquidity Providers to burn their LP tokens to redeem their underlying assets, including accumulated trading fees, proportional to their pool share.

## 🛠 Tech Stack

* **Smart Contract Logic:** Rust, Solana Program Library (SPL)
* **Framework:** Anchor (v0.30.1 / v1.2.0 compatibility)
* **Testing Infrastructure:** LiteSVM (blazing-fast, in-memory local testing via native Rust)

## 📂 Project Structure

```text
├── programs/amm-contract/
│   ├── src/
│   │   ├── instructions/
│   │   │   ├── deposit.rs         # Liquidity provision logic
│   │   │   ├── initialize.rs      # Pool creation logic
│   │   │   ├── mod.rs             # Module exports
│   │   │   ├── swap.rs            # Constant product swap logic
│   │   │   └── withdraw.rs        # Liquidity redemption logic
│   │   ├── errors.rs              # Custom error codes
│   │   ├── lib.rs                 # Program entrypoint
│   │   └── state.rs               # AmmPool account structure
│   ├── tests/
│   │   └── test_initialize.rs     # End-to-end LiteSVM test suite
│   └── Cargo.toml                 # Program dependencies
├── .gitignore                     # Git ignore rules
├── .prettierignore                # Prettier formatting rules
├── Anchor.toml                    # Anchor workspace configuration
├── Cargo.lock                     # Dependency lockfile
├── Cargo.toml                     # Workspace dependencies
├── README.md                      # Project documentation
└── rust-toolchain.toml            # Rust version configuration
