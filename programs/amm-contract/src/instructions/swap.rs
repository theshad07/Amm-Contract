use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{state::AmmPool, errors::AmmError};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, AmmPool>,

    #[account(mut)]
    pub vault_in: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_out: Account<'info, TokenAccount>,

    #[account(mut)]
    pub creator_ata_in: Account<'info, TokenAccount>,

    #[account(mut)]
    pub creator_ata_out: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

}

pub fn process_swap(ctx: Context<Swap>, amount: u64, min_amount_out: u64) -> Result<()> {

    require!(amount > 0, AmmError::ZeroInitialDeposit);
    
    let pool = &ctx.accounts.pool;
    let reserve_in = ctx.accounts.vault_in.amount as u128;
    let reserve_out = ctx.accounts.vault_out.amount as u128;

    require!(reserve_in > 0 && reserve_out > 0, AmmError::ZeroLiquidityMinted);

    let fee_factor = 10_000u128
    .checked_sub(pool.fee_points as u128)
    .ok_or(AmmError::MathOverflow)?;


    let amount_in_with_fee = (amount as u128)
    .checked_mul(fee_factor)
    .ok_or(AmmError::MathOverflow)?;

    let numerator = amount_in_with_fee
    .checked_mul(reserve_out)
    .ok_or(AmmError::MathOverflow)?;

    let denominator = reserve_in
    .checked_mul(10_000)
    .ok_or(AmmError::MathOverflow)?
    .checked_add(amount_in_with_fee)
    .ok_or(AmmError::MathOverflow)?;

    let amount_out = (numerator / denominator) as u64;

    require!(amount_out >= min_amount_out, AmmError::InvalidFee);

    let cpi_transfer_in = CpiContext::new( ctx.accounts.token_program.key(),
Transfer {
    from: ctx.accounts.creator_ata_in.to_account_info(),
    to: ctx.accounts.vault_in.to_account_info(),
    authority: ctx.accounts.creator.to_account_info(),
},
);
token::transfer(cpi_transfer_in, amount)?;

let mint_a_key = pool.mint_a.key();
let mint_b_key = pool.mint_b.key();

let signer_seeds: &[&[&[u8]]] = &[&[
    b"amm_pool",
    mint_a_key.as_ref(),
    mint_b_key.as_ref(),
    &[pool.bump],
]];

let cpi_transfer_out = CpiContext::new_with_signer(
    ctx.accounts.token_program.key(),
    Transfer{
        from: ctx.accounts.vault_out.to_account_info(),
        to: ctx.accounts.creator_ata_out.to_account_info(),
        authority: ctx.accounts.pool.to_account_info(),
    },
    signer_seeds
);

token::transfer(cpi_transfer_out, amount_out)?;

Ok(())


}
