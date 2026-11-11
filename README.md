# Turbin3 Vault and Escrow Programs

Solana programs built with Rust and Anchor as part of the Turbin3 assignment.

## Vault Program

Program ID: `3uWsJKD8QL6yjfZoHkESwAMe8GCXxv3pEtv1rSeU9j2s`

The vault supports:
- Initialization
- SOL deposits
- SOL withdrawals
- Vault closure and recovery of remaining SOL

PDA seeds:
- State: `[b"state", authority]`
- Vault: `[b"vault", authority]`

## Escrow Program

Program ID: `9bdAjbxL2EGG2zeA3P42cAAGrmrsm7doyhqmNGVxxt7P`

The escrow supports:
- `make`: Create an escrow and deposit Token A.
- `take`: Exchange Token A for Token B.
- `refund`: Return deposited Token A to the maker.
- `update`: Change the required amount of Token B.

PDA seed: `[b"escrow", maker]`

## Technologies

- Rust
- Solana
- Anchor
- TypeScript
- Solana Web3.js
- SPL Token
- Mocha and Chai

## Build and Test

Install dependencies:

`npm install`

Build:

`anchor build`

Run tests:

`anchor test --validator legacy`

## Test Coverage

The test suite covers both programs.

Vault:
- Initialization and deposits
- Successful withdrawals
- Zero-value deposits and withdrawals
- Unauthorized withdrawals
- Withdrawals exceeding available funds
- Closing the vault

Escrow:
- Creating an escrow
- Updating the required Token B amount
- Rejecting a zero-value update
- Completing a token exchange
- Refunding deposited tokens

Latest test result: **13 passing tests**.

## Current Limitations

- Each maker can have only one active escrow at a time because the escrow PDA is derived from the maker's public key.
- The maker must have an associated Token B account before the taker completes the exchange.
- The escrow uses the standard SPL Token Program.

## What I Learned

This project helped me understand account management, PDA derivation, program-controlled accounts, token transfers, CPI calls, Anchor constraints, and TypeScript testing.

## Links

- GitHub repository: https://github.com/Paulseun590/turbin3-vault-escrow
- Project explanation: https://x.com/Paulseun5/status/2120581861533794360
