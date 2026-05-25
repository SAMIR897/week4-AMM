use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::state::Config;
use crate::error::AmmError;

#[derive(Accounts)]
#[instruction(seed: u64, fee_basis_points: u16)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        space = Config::INIT_SPACE,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = initializer,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"lp", config.key().as_ref()],
        bump
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        token::mint = mint_x,
        token::authority = config,
        seeds = [b"vault_x", config.key().as_ref()],
        bump
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = initializer,
        token::mint = mint_y,
        token::authority = config,
        seeds = [b"vault_y", config.key().as_ref()],
        bump
    )]
    pub vault_y: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, seed: u64, fee_basis_points: u16, bumps: &InitializeBumps) -> Result<()> {
        require!(fee_basis_points <= 10000, AmmError::InvalidFee);

        self.config.set_inner(Config {
            seed,
            authority: Some(self.initializer.key()),
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            fee_basis_points,
            locked: false,
            config_bump: bumps.config,
            lp_bump: bumps.mint_lp,
            fee_x_pending: 0,
            fee_y_pending: 0,
        });

        Ok(())
    }
}
