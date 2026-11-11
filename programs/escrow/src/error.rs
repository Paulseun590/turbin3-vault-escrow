use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Token amounts must be greater than zero")]
    InvalidAmount,

    #[msg("The two token mints must be different")]
    InvalidMints,
}
