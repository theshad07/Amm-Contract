use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, Burn};
use crate::{state::AmmPool, errors::AmmError};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    #[account(mut, has_one = lp_mint)]
    pub pool: Account<'info, AmmPool>,

    #[account(mut)] 
    pub lp_mint: Account<'info, Mint>,

    #[account(mut, address = pool.vault_a)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut, address = pool.vault_b)]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer_ata_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer_ata_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer_lp_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn process_withdraw(ctx: Context<Withdraw>, lp_amount: u64, min_amount: u64) -> Result<()> {
    require!(lp_amount > 0 , AmmError::ZeroLiquidityMinted);

    let pool = &ctx.accounts.pool;
    
    let total_lp_supply = ctx.accounts.lp_mint.supply; 

  
    let amount_a = (lp_amount as u128)
        .checked_mul(ctx.accounts.vault_a.amount as u128)
        .unwrap()
        .checked_div(total_lp_supply as u128)
        .unwrap() as u64;

    let amount_b = (lp_amount as u128)
        .checked_mul(ctx.accounts.vault_b.amount as u128)
        .unwrap()
        .checked_div(total_lp_supply as u128)
        .unwrap() as u64;

    require!(amount_a > 0 && amount_b > 0, AmmError::ZeroLiquidityMinted);
    require!(amount_a >= min_amount && amount_b >= min_amount, AmmError::InvalidFee); 

   
    let cpi_burn = CpiContext::new(
        ctx.accounts.token_program.key(),
        Burn {
            mint: ctx.accounts.lp_mint.to_account_info(),
            from: ctx.accounts.withdrawer_lp_ata.to_account_info(),
            authority: ctx.accounts.withdrawer.to_account_info(),
        },
    );
    token::burn(cpi_burn, lp_amount)?;

    let mint_a_key = pool.mint_a.key();
    let mint_b_key = pool.mint_b.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"amm_pool",
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[pool.bump],
    ]];

   
    let cpi_transfer_a = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.vault_a.to_account_info(),
            to: ctx.accounts.withdrawer_ata_a.to_account_info(),
            authority: pool.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_transfer_a, amount_a)?;

    
    let cpi_transfer_b = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.vault_b.to_account_info(),
            to: ctx.accounts.withdrawer_ata_b.to_account_info(),
            authority: pool.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_transfer_b, amount_b)?;

    Ok(())
}