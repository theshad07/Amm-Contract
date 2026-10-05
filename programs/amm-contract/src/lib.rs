
pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use withdraw::*;

declare_id!("9XKn4NPug93QgEPneXbx778297K4jBYxJkvWTt5Eq9pt");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_points: u32) -> Result<()> {
        process_initialize(ctx, fee_points)
    }

    pub fn deposit(ctx: Context<Deposit>, amount_a: u64, amount_b: u64) -> Result<()> {
        process_deposit(ctx, amount_a, amount_b)
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        process_swap(ctx, amount_in, min_amount_out)
    }
    
    pub fn withdraw(ctx: Context<Withdraw> , lp_amount: u64,min_amount: u64) -> Result<()> {
        process_withdraw(ctx, lp_amount, min_amount )
    }
}
