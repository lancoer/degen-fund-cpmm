import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  createInitializeMetadataPointerInstruction,
  createInitializeMintInstruction,
  ExtensionType,
  getMintLen,
  getOrCreateAssociatedTokenAccount,
  LENGTH_SIZE,
  mintToChecked,
  TOKEN_2022_PROGRAM_ID,
  TYPE_SIZE,
} from "@solana/spl-token";
import { pack, TokenMetadata } from "@solana/spl-token-metadata";

import secret from "../test-ledger/id.json";

const mint_bte = async () => {
  const connection = new Connection(clusterApiUrl("devnet"), "confirmed");
  const wallet = Keypair.fromSecretKey(new Uint8Array(secret));

  // Generate new keypair for Mint Account
  const mintKeypair = Keypair.generate();
  // Address for Mint Account
  const mint = mintKeypair.publicKey;
  // Authority that can mint new tokens
  const mintAuthority = wallet.publicKey;
  // Authority that can update the metadata pointer and token metadata
  const updateAuthority = mintAuthority;
  const freezeAuthority = mintAuthority;

  const metadata: TokenMetadata = {
    mint,
    name: "Best Token Ever",
    symbol: "BTE",
    uri: "https://ipfs.io/ipfs/QmccsQSw2SZ5CZthrZJZJSL6YwFKucHJRUwnJhejJDdPW2",
    additionalMetadata: [],
  };

  // Size of Mint Account with extension
  const mintLen = getMintLen([ExtensionType.MetadataPointer]);
  // Size of MetadataExtension 2 bytes for type, 2 bytes for length
  const metadataExtension = TYPE_SIZE + LENGTH_SIZE;
  // Size of metadata
  const metadataLen = pack(metadata).length;
  // Minimum lamports required for Mint Account
  const lamports = await connection.getMinimumBalanceForRentExemption(
    mintLen + metadataExtension + metadataLen
  );

  // Instruction to invoke System Program to create new account
  const createAccountInstruction = SystemProgram.createAccount({
    fromPubkey: wallet.publicKey, // Account that will transfer lamports to created account
    newAccountPubkey: mint, // Address of the account to create
    space: mintLen, // Amount of bytes to allocate to the created account
    lamports, // Amount of lamports transferred to created account
    programId: TOKEN_2022_PROGRAM_ID, // Program assigned as owner of created account
  });

  // Instruction to initialize the MetadataPointer Extension
  const initializeMetadataPointerInstruction =
    createInitializeMetadataPointerInstruction(
      mint, // Mint Account address
      mintAuthority,
      mint,
      TOKEN_2022_PROGRAM_ID // Token Extension Program ID
    );

  // Instruction to initialize Mint Account data
  const initializeMintInstruction = createInitializeMintInstruction(
    mint, // Mint Account Address
    6, // Decimals of Mint
    updateAuthority, // Designated Mint Authority
    freezeAuthority, // Optional Freeze Authority
    TOKEN_2022_PROGRAM_ID // Token Extension Program ID
  );

  // Add instructions to new transaction
  const transaction = new Transaction().add(
    createAccountInstruction,
    initializeMetadataPointerInstruction,
    initializeMintInstruction
  );

  // Send transaction
  const transactionSignature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet, mintKeypair] // Signers
  );

  console.log(
    "\nCreate Mint Account:",
    `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
  );

  const token = await getOrCreateAssociatedTokenAccount(
    connection,
    wallet,
    mint,
    wallet.publicKey,
    false,
    undefined,
    undefined,
    TOKEN_2022_PROGRAM_ID
  );
  console.log(token.address.toString());

  await mintToChecked(
    connection,
    wallet,
    mint,
    token.address,
    updateAuthority,
    100_000_000_000_000,
    6,
    undefined,
    undefined,
    TOKEN_2022_PROGRAM_ID
  );

  console.log("Successfully minted 100 million tokens (", mint.toString(), ")");
};

mint_bte();
