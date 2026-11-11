use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};

use crate::{constants::ESCROW_SEED, state::EscrowState};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub mint_a: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, maker.key().as_ref()],
        bump = escrow.bump,
        has_one = maker,
        has_one = mint_a,
        close = maker
    )]
    pub escrow: Account<'info, EscrowState>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_refund(ctx: Context<Refund>) -> Result<()> {
    let maker_key = ctx.accounts.maker.key();
    let bump = [ctx.accounts.escrow.bump];

    let signer_seeds: &[&[u8]] = &[ESCROW_SEED, maker_key.as_ref(), &bump];
    let signer = &[signer_seeds];

    let transfer_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.maker_ata_a.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };

    let transfer_context =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), transfer_accounts, signer);

    token::transfer(transfer_context, ctx.accounts.escrow.amount_a)?;

    let close_accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(),
        destination: ctx.accounts.maker.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };

    let close_context =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), close_accounts, signer);

    token::close_account(close_context)?;

    msg!("Escrow refunded to maker");
    Ok(())
}
