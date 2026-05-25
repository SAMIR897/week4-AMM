use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, burn, transfer, Burn, Transfer};
use crate::state::Config;
use crate::error::AmmError;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Box<Account<'info, Mint>>,

    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,

    #[account(mut, token::mint = mint_x)]
    pub user_x: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = mint_y)]
    pub user_y: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = mint_lp)]
    pub user_lp: Box<Account<'info, TokenAccount>>,

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

    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);

        let amount_x = (amount as u128)
            .checked_mul(self.vault_x.amount as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(self.mint_lp.supply as u128)
            .ok_or(AmmError::MathOverflow)? as u64;

        let amount_y = (amount as u128)
            .checked_mul(self.vault_y.amount as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(self.mint_lp.supply as u128)
            .ok_or(AmmError::MathOverflow)? as u64;

        require!(amount_x >= min_x && amount_y >= min_y, AmmError::SlippageExceeded);

        self.burn_lp_tokens(amount)?;
        self.transfer_from_vault(amount_x, true)?;
        self.transfer_from_vault(amount_y, false)?;

        Ok(())
    }

    fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.key(), cpi_accounts);
        burn(cpi_ctx, amount)
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
            (self.vault_x.to_account_info(), self.user_x.to_account_info())
        } else {
            (self.vault_y.to_account_info(), self.user_y.to_account_info())
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
