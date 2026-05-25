pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod cpmm;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HJXxe95ehuygYvffAJDU6B9Z2s9JdfQVtS7KZJMobiCD");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: u64, fee_basis_points: u16) -> Result<()> {
        ctx.accounts.init(seed, fee_basis_points, &ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y)
    }

    pub fn swap(ctx: Context<Swap>, is_x: bool, amount_in: u64, min_amount_out: u64) -> Result<()> {
        ctx.accounts.swap(is_x, amount_in, min_amount_out)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, fee_basis_points: Option<u16>, locked: Option<bool>, new_authority: Option<Pubkey>) -> Result<()> {
        ctx.accounts.update(fee_basis_points, locked, new_authority)
    }

    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        ctx.accounts.collect()
    }
}
