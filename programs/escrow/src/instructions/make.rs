use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

use crate::{constants::ESCROW_SEED, error::EscrowError, state::EscrowState};

#[derive(Accounts)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = 8 + EscrowState::INIT_SPACE,
        seeds = [ESCROW_SEED, maker.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, EscrowState>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_make(ctx: Context<Make>, amount_a: u64, amount_b: u64) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, EscrowError::InvalidAmount);
    require_keys_neq!(
        ctx.accounts.mint_a.key(),
        ctx.accounts.mint_b.key(),
        EscrowError::InvalidMints
    );

    ctx.accounts.escrow.maker = ctx.accounts.maker.key();
    ctx.accounts.escrow.mint_a = ctx.accounts.mint_a.key();
    ctx.accounts.escrow.mint_b = ctx.accounts.mint_b.key();
    ctx.accounts.escrow.amount_a = amount_a;
    ctx.accounts.escrow.amount_b = amount_b;
    ctx.accounts.escrow.bump = ctx.bumps.escrow;

    let cpi_accounts = Transfer {
        from: ctx.accounts.maker_ata_a.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);

    token::transfer(cpi_context, amount_a)?;

    msg!("Escrow created. Deposited {} units of Token A", amount_a);
    Ok(())
}
