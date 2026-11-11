use anchor_lang::prelude::*;

use crate::{constants::*, error::VaultError, state::VaultState};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [STATE_SEED, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, authority.key().as_ref()],
        bump = state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.authority.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);

    anchor_lang::system_program::transfer(cpi_context, amount)?;

    msg!("Deposited {} lamports", amount);
    Ok(())
}
