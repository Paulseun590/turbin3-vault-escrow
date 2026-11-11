pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("9bdAjbxL2EGG2zeA3P42cAAGrmrsm7doyhqmNGVxxt7P");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, amount_a: u64, amount_b: u64) -> Result<()> {
        instructions::make::handle_make(ctx, amount_a, amount_b)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::handle_take(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::handle_refund(ctx)
    }

    pub fn update(ctx: Context<Update>, new_amount_b: u64) -> Result<()> {
        instructions::update::handle_update(ctx, new_amount_b)
    }
}
