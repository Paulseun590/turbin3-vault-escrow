use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};

use crate::{constants::ESCROW_SEED, state::EscrowState};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: This address is checked against the maker stored in escrow.
    #[account(mut, address = escrow.maker)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, maker.key().as_ref()],
        bump = escrow.bump,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        close = maker
    )]
    pub escrow: Box<Account<'info, EscrowState>>,

    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = taker
    )]
    pub taker_ata_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker
    )]
    pub taker_ata_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = maker
    )]
    pub maker_ata_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_take(ctx: Context<Take>) -> Result<()> {
    let maker_key = ctx.accounts.maker.key();
    let bump = [ctx.accounts.escrow.bump];

    let signer_seeds: &[&[u8]] = &[ESCROW_SEED, maker_key.as_ref(), &bump];
    let signer = &[signer_seeds];

    // First, the taker pays the maker the requested amount of Token B.
    let pay_accounts = Transfer {
        from: ctx.accounts.taker_ata_b.to_account_info(),
        to: ctx.accounts.maker_ata_b.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };

    let pay_context = CpiContext::new(ctx.accounts.token_program.key(), pay_accounts);

    token::transfer(pay_context, ctx.accounts.escrow.amount_b)?;

    // Then, the escrow sends the deposited Token A to the taker.
    let release_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.taker_ata_a.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };

    let release_context =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), release_accounts, signer);

    token::transfer(release_context, ctx.accounts.escrow.amount_a)?;

    // Close the now-empty vault token account and return its rent to the maker.
    let close_accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(),
        destination: ctx.accounts.maker.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };

    let close_context =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), close_accounts, signer);

    token::close_account(close_context)?;

    msg!("Escrow completed");
    Ok(())
}
