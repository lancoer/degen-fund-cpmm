import { ComputeBudgetProgram, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

import { DegenFundCpmm } from "../target/types/degen_fund_cpmm";
import { findProgramAddress, getCreatePoolKeys } from "./utils/pda";

const RAYDIUM_CP_SWAP = new PublicKey(
  "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW"
);
// const RAYDIUM_CP_SWAP = new PublicKey("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

const WSOL_MINT = new PublicKey("So11111111111111111111111111111111111111112");
const BTE_MINT = new PublicKey("7P9GFzJMsVFwt6QxGckhikhqypPETqdhXujjrLWxHYRx");

describe("degen_fund_cpmm", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DegenFundCpmm as Program<DegenFundCpmm>;
  const connection = provider.connection;
  const wallet = provider.wallet;

  it("create pool", async () => {
    const slot = await connection.getSlot();
    const openTime = await connection.getBlockTime(slot);
    const poolKeys = getCreatePoolKeys({
      programId: RAYDIUM_CP_SWAP,
      mintA: WSOL_MINT,
      mintB: BTE_MINT,
    });
    const tx = await program.methods
      .initialize(
        new anchor.BN(1_000),
        new anchor.BN(100_000_000_000),
        new anchor.BN(openTime)
      )
      .accounts({
        creator: wallet.publicKey,
        ammConfig: poolKeys.configId,
        token0Mint: WSOL_MINT,
        token1Mint: BTE_MINT,
        creatorLpToken: findProgramAddress(
          [
            wallet.publicKey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            poolKeys.lpMint.toBuffer(),
          ],
          ASSOCIATED_TOKEN_PROGRAM_ID
        ).publicKey,
        creatorToken0: getAssociatedTokenAddressSync(
          WSOL_MINT,
          wallet.publicKey
        ),
        creatorToken1: getAssociatedTokenAddressSync(
          BTE_MINT,
          wallet.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        ),
        token0Program: TOKEN_PROGRAM_ID,
        token1Program: TOKEN_2022_PROGRAM_ID,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 100_000 }),
        ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
      ])
      .rpc({ skipPreflight: true });
    console.log("Your transaction signature", tx);
  });
});
