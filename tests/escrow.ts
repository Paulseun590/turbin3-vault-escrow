import * as anchor from "@anchor-lang/core";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import escrowIdl from "../target/idl/escrow.json";

const BN = require("bn.js");

describe("Escrow Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program: any = new anchor.Program(
    escrowIdl as anchor.Idl,
    provider
  );

  const connection = provider.connection;
  const payer = (provider.wallet as any).payer as Keypair;

  const amountA = 1_000_000;
  const amountB = 2_000_000;
  const updatedAmountB = 1_500_000;

  let mintA: PublicKey;
  let mintB: PublicKey;

  let makerTake: Keypair;
  let taker: Keypair;
  let makerRefund: Keypair;

  let makerTakeAtaA: PublicKey;
  let makerTakeAtaB: PublicKey;
  let takerAtaA: PublicKey;
  let takerAtaB: PublicKey;
  let makerRefundAtaA: PublicKey;

  let takeEscrow: PublicKey;
  let takeVault: PublicKey;
  let refundEscrow: PublicKey;
  let refundVault: PublicKey;

  async function fundAccount(keypair: Keypair) {
    const signature = await connection.requestAirdrop(
      keypair.publicKey,
      3 * LAMPORTS_PER_SOL
    );

    const latest = await connection.getLatestBlockhash();

    await connection.confirmTransaction(
      {
        signature,
        ...latest,
      },
      "confirmed"
    );
  }

  async function expectFailure(action: () => Promise<unknown>) {
    let thrown: unknown;

    try {
      await action();
    } catch (error) {
      thrown = error;
    }

    expect(thrown, "Expected transaction to fail").to.not.equal(undefined);
  }

  function deriveEscrow(maker: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.toBuffer()],
      program.programId
    )[0];
  }

  before(async () => {
    makerTake = Keypair.generate();
    taker = Keypair.generate();
    makerRefund = Keypair.generate();

    await Promise.all([
      fundAccount(makerTake),
      fundAccount(taker),
      fundAccount(makerRefund),
    ]);

    mintA = await createMint(
      connection,
      payer,
      payer.publicKey,
      null,
      6
    );

    mintB = await createMint(
      connection,
      payer,
      payer.publicKey,
      null,
      6
    );

    const makerTakeA = await getOrCreateAssociatedTokenAccount(
      connection, payer, mintA, makerTake.publicKey
    );
    const makerTakeB = await getOrCreateAssociatedTokenAccount(
      connection, payer, mintB, makerTake.publicKey
    );
    const takerA = await getOrCreateAssociatedTokenAccount(
      connection, payer, mintA, taker.publicKey
    );
    const takerB = await getOrCreateAssociatedTokenAccount(
      connection, payer, mintB, taker.publicKey
    );
    const makerRefundA = await getOrCreateAssociatedTokenAccount(
      connection, payer, mintA, makerRefund.publicKey
    );

    makerTakeAtaA = makerTakeA.address;
    makerTakeAtaB = makerTakeB.address;
    takerAtaA = takerA.address;
    takerAtaB = takerB.address;
    makerRefundAtaA = makerRefundA.address;

    await mintTo(
      connection, payer, mintA, makerTakeAtaA, payer, 5_000_000
    );
    await mintTo(
      connection, payer, mintA, makerRefundAtaA, payer, 3_000_000
    );
    await mintTo(
      connection, payer, mintB, takerAtaB, payer, 5_000_000
    );

    takeEscrow = deriveEscrow(makerTake.publicKey);
    refundEscrow = deriveEscrow(makerRefund.publicKey);

    takeVault = getAssociatedTokenAddressSync(
      mintA, takeEscrow, true
    );
    refundVault = getAssociatedTokenAddressSync(
      mintA, refundEscrow, true
    );
  });

  it("Creates an escrow and deposits Token A", async () => {
    await program.methods
      .make(new BN(amountA), new BN(amountB))
      .accounts({
        maker: makerTake.publicKey,
        mintA,
        mintB,
        makerAtaA: makerTakeAtaA,
        escrow: takeEscrow,
        vault: takeVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([makerTake])
      .rpc();

    const vaultAccount = await getAccount(connection, takeVault);
    const makerAccount = await getAccount(connection, makerTakeAtaA);

    expect(Number(vaultAccount.amount)).to.equal(amountA);
    expect(Number(makerAccount.amount)).to.equal(4_000_000);
  });

  it("Updates the required Token B amount", async () => {
    await program.methods
      .update(new BN(updatedAmountB))
      .accounts({
        maker: makerTake.publicKey,
        escrow: takeEscrow,
      })
      .signers([makerTake])
      .rpc();

    const state: any = await program.account.escrowState.fetch(takeEscrow);

    expect(Number(state.amountB.toString())).to.equal(updatedAmountB);
  });

  it("Rejects an update with zero Token B", async () => {
    await expectFailure(() =>
      program.methods
        .update(new BN(0))
        .accounts({
          maker: makerTake.publicKey,
          escrow: takeEscrow,
        })
        .signers([makerTake])
        .rpc()
    );
  });

  it("Completes the exchange and closes the escrow", async () => {
    await program.methods
      .take()
      .accounts({
        taker: taker.publicKey,
        maker: makerTake.publicKey,
        escrow: takeEscrow,
        mintA,
        mintB,
        takerAtaA: takerAtaA,
        takerAtaB: takerAtaB,
        makerAtaB: makerTakeAtaB,
        vault: takeVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    const takerA = await getAccount(connection, takerAtaA);
    const takerB = await getAccount(connection, takerAtaB);
    const makerB = await getAccount(connection, makerTakeAtaB);

    expect(Number(takerA.amount)).to.equal(amountA);
    expect(Number(takerB.amount)).to.equal(5_000_000 - updatedAmountB);
    expect(Number(makerB.amount)).to.equal(updatedAmountB);

    expect(await connection.getAccountInfo(takeEscrow)).to.equal(null);
    expect(await connection.getAccountInfo(takeVault)).to.equal(null);
  });

  it("Refunds Token A and closes the escrow", async () => {
    await program.methods
      .make(new BN(500_000), new BN(800_000))
      .accounts({
        maker: makerRefund.publicKey,
        mintA,
        mintB,
        makerAtaA: makerRefundAtaA,
        escrow: refundEscrow,
        vault: refundVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([makerRefund])
      .rpc();

    const vaultBeforeRefund = await getAccount(connection, refundVault);
    expect(Number(vaultBeforeRefund.amount)).to.equal(500_000);

    // Record the maker's balance after depositing Token A.
    const makerBalanceBefore = Number(
      (await getAccount(connection, makerRefundAtaA)).amount
    );

    await program.methods
      .refund()
      .accounts({
        maker: makerRefund.publicKey,
        mintA,
        makerAtaA: makerRefundAtaA,
        escrow: refundEscrow,
        vault: refundVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([makerRefund])
      .rpc();

    const makerBalanceAfter = Number(
      (await getAccount(connection, makerRefundAtaA)).amount
    );

    expect(makerBalanceAfter - makerBalanceBefore).to.equal(500_000);
    expect(await connection.getAccountInfo(refundEscrow)).to.equal(null);
    expect(await connection.getAccountInfo(refundVault)).to.equal(null);
  });
});
