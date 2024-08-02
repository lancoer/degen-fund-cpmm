import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";
import {
  createInitializeMintCloseAuthorityInstruction,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";

import secret from "../test-ledger/id.json";

const add_ext = async () => {
  const connection = new Connection(clusterApiUrl("devnet"), "confirmed");
  const wallet = Keypair.fromSecretKey(new Uint8Array(secret));

  // Address for Mint Account
  const mint = new PublicKey("Eat2Gpa1jKRBqXNjZsSo4cNLxnBfoeKAMuH2BYmVintR");
  // Authority that can close minting new tokens
  const closeAuthority = wallet.publicKey;

  // Instruction to initialize the MintCloseAuthority Extension
  const initializeMintCloseAuthorityInstruction =
    createInitializeMintCloseAuthorityInstruction(
      mint,
      closeAuthority,
      TOKEN_2022_PROGRAM_ID // Token Extension Program ID
    );

  // Add instructions to new transaction
  const transaction = new Transaction().add(
    initializeMintCloseAuthorityInstruction
  );

  // Send transaction
  const transactionSignature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet] // Signers
  );

  console.log(
    `\nAdded  MintCloseAuthority extension to token ${mint.toString()}:`,
    `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
  );
};

add_ext();
