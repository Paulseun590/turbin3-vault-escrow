use anchor_lang::prelude::*;

use crate::{constants::*, error::VaultError, state::VaultState};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

pub fn handle_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let rent_reserve = Rent::get()?.minimum_balance(0);
    let vault_balance = ctx.accounts.vault.lamports();

    let available_balance = vault_balance
        .checked_sub(rent_reserve)
        .ok_or(VaultError::InsufficientFunds)?;

    require!(amount <= available_balance, VaultError::InsufficientFunds);

    let authority_key = ctx.accounts.authority.key();
    let bump = [ctx.accounts.state.vault_bump];

    let signer_seeds: &[&[u8]] = &[VAULT_SEED, authority_key.as_ref(), &bump];

    let signer = &[signer_seeds];

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.authority.to_account_info(),
    };

    let cpi_context =
        CpiContext::new_with_signer(ctx.accounts.system_program.key(), cpi_accounts, signer);

    anchor_lang::system_program::transfer(cpi_context, amount)?;

    msg!("Withdrew {} lamports", amount);
    Ok(())
}
