use anchor_lang::prelude::*;

#[account]
pub struct AmmPool {
    pub creator: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub fee_points: u32,
    pub bump: u8,
}

impl AmmPool {
    pub const INIT_SPACE: usize = (6 * 32) + 4 + 1;
}