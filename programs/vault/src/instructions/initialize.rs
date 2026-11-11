use anchor_lang::prelude::*;

use crate::{constants::*, state::VaultState};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [STATE_SEED, authority.key().as_ref()],
        bump
    )]
    pub state: Account<'info, VaultState>,

    /// CHECK: This is a PDA used as a system-owned SOL vault.
    /// Its address is verified by the seeds constraint.
    #[account(
        mut,
        seeds = [VAULT_SEED, authority.key().as_ref()],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.state.authority = ctx.accounts.authority.key();
    ctx.accounts.state.vault_bump = ctx.bumps.vault;

    let rent_reserve = Rent::get()?.minimum_balance(0);

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.authority.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);

    anchor_lang::system_program::transfer(cpi_context, rent_reserve)?;

    msg!("Vault initialized");
    Ok(())
}
