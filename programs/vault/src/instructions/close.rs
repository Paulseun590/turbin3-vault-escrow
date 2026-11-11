use anchor_lang::prelude::*;

use crate::{constants::*, state::VaultState};

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        close = authority,
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

pub fn handle_close(ctx: Context<Close>) -> Result<()> {
    let vault_balance = ctx.accounts.vault.lamports();

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

    anchor_lang::system_program::transfer(cpi_context, vault_balance)?;

    msg!("Vault closed and remaining lamports returned");
    Ok(())
}
