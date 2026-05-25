use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Math operation overflow")]
    MathOverflow,
    #[msg("Fee basis points cannot exceed 10000")]
    InvalidFee,
    #[msg("Pool is currently locked")]
    PoolLocked,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Invalid authority")]
    InvalidAuthority,
}
