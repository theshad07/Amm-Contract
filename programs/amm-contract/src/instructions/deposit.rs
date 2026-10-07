use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use crate::{state::AmmPool, errors::AmmError};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, AmmPool>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositor_ata_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositor_ata_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositor_lp_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn process_deposit(ctx: Context<Deposit>, amount_a: u64, amount_b: u64) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, AmmError::ZeroInitialDeposit);

    require_keys_eq!(ctx.accounts.pool.lp_mint, ctx.accounts.lp_mint.key());
    require_keys_eq!(ctx.accounts.vault_a.key(), ctx.accounts.pool.vault_a);
    require_keys_eq!(ctx.accounts.vault_b.key(), ctx.accounts.pool.vault_b);

    let pool = &ctx.accounts.pool;
    let total_lp_supply = ctx.accounts.lp_mint.supply;

   
    let lp_to_mint: u64 = if total_lp_supply == 0 {
     
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(AmmError::MathOverflow)?;
        integer_sqrt(product) as u64
    } else {
       
        let share_a = (amount_a as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(ctx.accounts.vault_a.amount as u128)
            .ok_or(AmmError::MathOverflow)?;

        let share_b = (amount_b as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(ctx.accounts.vault_b.amount as u128)
            .ok_or(AmmError::MathOverflow)?;

        std::cmp::min(share_a, share_b) as u64
    };

    require!(lp_to_mint > 0, AmmError::ZeroLiquidityMinted);

    
    let cpi_transfer_a = CpiContext::new(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.depositor_ata_a.to_account_info(),
            to: ctx.accounts.vault_a.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        },
    );
    token::transfer(cpi_transfer_a, amount_a)?;

   
    let cpi_transfer_b = CpiContext::new(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.depositor_ata_b.to_account_info(),
            to: ctx.accounts.vault_b.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        },
    );
    token::transfer(cpi_transfer_b, amount_b)?;

    
    let mint_a_key = pool.mint_a.key();
    let mint_b_key = pool.mint_b.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"amm_pool",
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[pool.bump],
    ]];

    let cpi_mint_lp = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        MintTo {
            mint: ctx.accounts.lp_mint.to_account_info(),
            to: ctx.accounts.depositor_lp_ata.to_account_info(),
            authority: pool.to_account_info(),
        },
        signer_seeds,
    );
    token::mint_to(cpi_mint_lp, lp_to_mint)?;

    Ok(())
}


fn integer_sqrt(val: u128) -> u128 {
    if val == 0 {
        return 0;
    }
    let mut x0 = val / 2;
    if x0 == 0 {
        return 1;
    }
    let mut x1 = (x0 + val / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + val / x0) / 2;
    }
    x0
}
