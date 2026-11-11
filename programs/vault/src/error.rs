use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("The amount must be greater than zero")]
    InvalidAmount,

    #[msg("The vault does not have enough withdrawable SOL")]
    InsufficientFunds,
}
