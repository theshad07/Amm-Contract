use anchor_lang::prelude::*;    

#[error_code]
pub enum AmmError {
    #[msg("Invalid fee configuration")]
    InvalidFee,
    #[msg("Math operation overflowed")]
    MathOverflow,             
    #[msg("Zero liquidity minted")]
    ZeroLiquidityMinted,      
    #[msg("Zero initial deposit is not allowed")]
    ZeroInitialDeposit,       
}