use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{state::AmmPool, errors::AmmError};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = 8 + AmmPool::INIT_SPACE,
        seeds = [b"amm_pool", mint_a.key().as_ref(),mint_b.key().as_ref()],
        bump
    )]
    pub pool: Account<'info,AmmPool>,

    #[account(
        init,
        payer = creator,
        seeds = [b"lp_mint", pool.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = pool,
    )]
    pub lp_mint: Account<'info,Mint>,

    #[account(
        init,
        payer = creator,
        seeds = [b"vault_a", pool.key().as_ref()],
        bump,
        token::mint = mint_a,
        token::authority = pool,

    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        seeds = [b"vault_b", pool.key().as_ref()],
        bump,
        token::mint = mint_b,
        token::authority = pool,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn process_initialize(ctx: Context<Initialize>,
fee_points: u32) -> Result<()> {
    require!(fee_points <= 10_000, AmmError::InvalidFee);

    let pool = &mut ctx.accounts.pool;
    pool.creator = ctx.accounts.creator.key();
    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.fee_points = fee_points;
    pool.bump = ctx.bumps.pool;

    Ok(())
}

