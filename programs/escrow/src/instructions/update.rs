use anchor_lang::prelude::*;

use crate::{constants::ESCROW_SEED, error::EscrowError, state::EscrowState};

#[derive(Accounts)]
pub struct Update<'info> {
    pub maker: Signer<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, maker.key().as_ref()],
        bump = escrow.bump,
        has_one = maker
    )]
    pub escrow: Account<'info, EscrowState>,
}

pub fn handle_update(ctx: Context<Update>, new_amount_b: u64) -> Result<()> {
    require!(new_amount_b > 0, EscrowError::InvalidAmount);

    ctx.accounts.escrow.amount_b = new_amount_b;

    msg!("Escrow requested amount updated to {}", new_amount_b);
    Ok(())
}
