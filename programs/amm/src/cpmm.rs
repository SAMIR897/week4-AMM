use anchor_lang::prelude::*;
use crate::error::AmmError;

pub fn calculate_swap_amount(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_basis_points: u16,
) -> Result<u64> {
    let amount_in_128 = amount_in as u128;
    let reserve_in_128 = reserve_in as u128;
    let reserve_out_128 = reserve_out as u128;
    let fee_multiplier = (10000 - fee_basis_points) as u128;

    let amount_in_with_fee = amount_in_128.checked_mul(fee_multiplier).ok_or(AmmError::MathOverflow)?;
    let numerator = amount_in_with_fee.checked_mul(reserve_out_128).ok_or(AmmError::MathOverflow)?;
    let denominator = reserve_in_128
        .checked_mul(10000)
        .ok_or(AmmError::MathOverflow)?
        .checked_add(amount_in_with_fee)
        .ok_or(AmmError::MathOverflow)?;

    let amount_out = numerator.checked_div(denominator).ok_or(AmmError::MathOverflow)?;

    Ok(amount_out as u64)
}

pub fn calculate_deposit_amounts(
    amount_x: u64,
    reserve_x: u64,
    reserve_y: u64,
) -> Result<u64> {
    let amount_x_128 = amount_x as u128;
    let reserve_x_128 = reserve_x as u128;
    let reserve_y_128 = reserve_y as u128;

    let amount_y = amount_x_128
        .checked_mul(reserve_y_128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_x_128)
        .ok_or(AmmError::MathOverflow)?;

    Ok(amount_y as u64)
}
