use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, transfer, Transfer};
use crate::state::Config;
use crate::error::AmmError;

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = config.authority == Some(authority.key()) @ AmmError::InvalidAuthority,
        has_one = mint_x,
        has_one = mint_y,
    )]
    pub config: Box<Account<'info, Config>>,

    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"vault_x", config.key().as_ref()],
        bump
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"vault_y", config.key().as_ref()],
        bump
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = mint_x)]
    pub treasury_x: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = mint_y)]
    pub treasury_y: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> CollectFees<'info> {
    pub fn collect(&mut self) -> Result<()> {
        let fee_x = self.config.fee_x_pending;
        let fee_y = self.config.fee_y_pending;

        if fee_x > 0 {
            self.transfer_from_vault(fee_x, true)?;
            self.config.fee_x_pending = 0;
        }

        if fee_y > 0 {
            self.transfer_from_vault(fee_y, false)?;
            self.config.fee_y_pending = 0;
        }

        Ok(())
    }

    fn transfer_from_vault(&self, amount: u64, is_x: bool) -> Result<()> {
        let seed_bytes = self.config.seed.to_le_bytes();
        let seeds = &[
            b"config",
            seed_bytes.as_ref(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let (from, to) = if is_x {
            (self.vault_x.to_account_info(), self.treasury_x.to_account_info())
        } else {
            (self.vault_y.to_account_info(), self.treasury_y.to_account_info())
        };

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)
    }
}
