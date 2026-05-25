use anchor_lang::prelude::*;
use crate::state::Config;
use crate::error::AmmError;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = config.authority == Some(authority.key()) @ AmmError::InvalidAuthority,
    )]
    pub config: Account<'info, Config>,
}

impl<'info> UpdateConfig<'info> {
    pub fn update(&mut self, fee_basis_points: Option<u16>, locked: Option<bool>, new_authority: Option<Pubkey>) -> Result<()> {
        if let Some(fee) = fee_basis_points {
            require!(fee <= 10000, AmmError::InvalidFee);
            self.config.fee_basis_points = fee;
        }

        if let Some(l) = locked {
            self.config.locked = l;
        }

        if let Some(auth) = new_authority {
            self.config.authority = Some(auth);
        }

        Ok(())
    }
}
