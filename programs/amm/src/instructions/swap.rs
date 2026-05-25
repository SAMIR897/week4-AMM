use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, transfer, Transfer};
use crate::state::Config;
use crate::error::AmmError;
use crate::cpmm::calculate_swap_amount;

#[derive(Accounts)]
pub struct Swap<'info> {
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

    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,

    #[account(mut, token::mint = mint_x)]
    pub user_x: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = mint_y)]
    pub user_y: Box<Account<'info, TokenAccount>>,

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

impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount_in: u64, min_amount_out: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);

        let (reserve_in, reserve_out) = if is_x {
            (self.vault_x.amount, self.vault_y.amount)
        } else {
            (self.vault_y.amount, self.vault_x.amount)
        };

        let amount_out = calculate_swap_amount(
            amount_in,
            reserve_in,
            reserve_out,
            self.config.fee_basis_points,
        )?;

        require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

        let fee_amount = (amount_in as u128)
            .checked_mul(self.config.fee_basis_points as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        if is_x {
            self.config.fee_x_pending = self.config.fee_x_pending.checked_add(fee_amount).unwrap();
        } else {
            self.config.fee_y_pending = self.config.fee_y_pending.checked_add(fee_amount).unwrap();
        }

        self.transfer_to_vault(amount_in, is_x)?;
        self.transfer_from_vault(amount_out, !is_x)?;

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
