use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, mint_to, transfer, MintTo, Transfer};
use crate::state::Config;
use crate::error::AmmError;

#[derive(Accounts)]
pub struct Deposit<'info> {
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

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);

        let (amount_x, amount_y) = if self.mint_lp.supply == 0 {
            (max_x, max_y)
        } else {
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

            (amount_x, amount_y)
        };

        require!(amount_x <= max_x && amount_y <= max_y, AmmError::SlippageExceeded);

        self.transfer_to_vault(amount_x, true)?;
        self.transfer_to_vault(amount_y, false)?;
        self.mint_lp_tokens(amount)?;

        Ok(())
    }

    fn transfer_to_vault(&self, amount: u64, is_x: bool) -> Result<()> {
        let (from, to) = if is_x {
            (self.user_x.to_account_info(), self.vault_x.to_account_info())
        } else {
            (self.user_y.to_account_info(), self.vault_y.to_account_info())
        };

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.key(), cpi_accounts);
        transfer(cpi_ctx, amount)
    }

    fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let seed_bytes = self.config.seed.to_le_bytes();
        let seeds = &[
            b"config",
            seed_bytes.as_ref(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, signer_seeds);
        mint_to(cpi_ctx, amount)
    }
}
