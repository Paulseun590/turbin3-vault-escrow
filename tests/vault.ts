import * as anchor from "@anchor-lang/core";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import vaultIdl from "../target/idl/vault.json";

const BN = require("bn.js");

describe("Vault Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program: any = new anchor.Program(
    vaultIdl as anchor.Idl,
    provider
  );

  const authority = provider.wallet.publicKey;

  const [statePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("state"), authority.toBuffer()],
    program.programId
  );

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), authority.toBuffer()],
    program.programId
  );

  const depositAmount = 1_000_000_000;
  const withdrawAmount = 400_000_000;

  async function expectFailure(
    action: () => Promise<unknown>,
    expectedError?: string
  ) {
    let thrown: unknown;

    try {
      await action();
    } catch (error) {
      thrown = error;
    }

    expect(thrown, "Expected transaction to fail").to.not.equal(undefined);

    if (expectedError) {
      expect(String(thrown)).to.include(expectedError);
    }
  }

  it("Initializes the vault and creates its state PDA", async () => {
    await program.methods
      .initialize()
      .accounts({
        authority,
        state: statePda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const stateAccount = await provider.connection.getAccountInfo(statePda);
    const vaultBalance = await provider.connection.getBalance(vaultPda);
    const rentReserve =
      await provider.connection.getMinimumBalanceForRentExemption(0);

    expect(stateAccount).to.not.equal(null);
    expect(vaultBalance).to.equal(rentReserve);
  });

  it("Deposits SOL into the vault", async () => {
    const balanceBefore =
      await provider.connection.getBalance(vaultPda);

    await program.methods
      .deposit(new BN(depositAmount))
      .accounts({
        authority,
        state: statePda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const balanceAfter =
      await provider.connection.getBalance(vaultPda);

    expect(balanceAfter - balanceBefore).to.equal(depositAmount);
  });

  it("Rejects a zero-value deposit", async () => {
    await expectFailure(() =>
      program.methods
        .deposit(new BN(0))
        .accounts({
          authority,
          state: statePda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
      "InvalidAmount"
    );
  });

  it("Rejects an unauthorized withdrawal", async () => {
    const unauthorized = Keypair.generate();

    await expectFailure(() =>
      program.methods
        .withdraw(new BN(100_000_000))
        .accounts({
          authority: unauthorized.publicKey,
          state: statePda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([unauthorized])
        .rpc()
    );
  });

  it("Withdraws SOL from the vault", async () => {
    const balanceBefore =
      await provider.connection.getBalance(vaultPda);

    await program.methods
      .withdraw(new BN(withdrawAmount))
      .accounts({
        authority,
        state: statePda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const balanceAfter =
      await provider.connection.getBalance(vaultPda);

    expect(balanceBefore - balanceAfter).to.equal(withdrawAmount);
  });

  it("Rejects a zero-value withdrawal", async () => {
    await expectFailure(() =>
      program.methods
        .withdraw(new BN(0))
        .accounts({
          authority,
          state: statePda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
      "InvalidAmount"
    );
  });

  it("Rejects a withdrawal exceeding available funds", async () => {
    await expectFailure(() =>
      program.methods
        .withdraw(new BN(10_000_000_000))
        .accounts({
          authority,
          state: statePda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
      "InsufficientFunds"
    );
  });

  it("Closes the vault and returns its remaining SOL", async () => {
    const balanceBefore =
      await provider.connection.getBalance(vaultPda);

    const authorityBalanceBefore =
      await provider.connection.getBalance(authority);

    await program.methods
      .close()
      .accounts({
        authority,
        state: statePda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const stateAccount =
      await provider.connection.getAccountInfo(statePda);

    const vaultBalance =
      await provider.connection.getBalance(vaultPda);

    const authorityBalanceAfter =
      await provider.connection.getBalance(authority);

    expect(stateAccount).to.equal(null);
    expect(vaultBalance).to.equal(0);

    // The authority receives the remaining vault balance.
    // The difference also includes the transaction fee.
    expect(authorityBalanceAfter).to.be.greaterThan(
      authorityBalanceBefore - 5_000
    );

    expect(balanceBefore).to.be.greaterThan(0);
  });
});
