// Calls the deployed spl_token_deploy program's create_token instruction to
// mint a brand new SPL token. Signs with DEPLOYER_PRIVATE_KEY from .env.
//
// Usage: yarn create-token   (after `anchor deploy`)
import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import idl from "../target/idl/spl_token_deploy.json";
import type { SplTokenDeploy } from "../target/types/spl_token_deploy";

const DECIMALS = 6; // change if you want a different precision
const NAME = "Example Token 1"; // <= 32 chars (Metaplex on-chain limit)
const SYMBOL = "EKT"; // <= 10 chars
// Off-chain JSON (image, description, etc.) - optional. Leave "" if you don't
// have one yet; on-chain name/symbol still show up in most wallets/explorers
// without it. Point this at a real hosted JSON later and call
// updateMetadataAccountV2 (with your own admin wallet - it's the update
// authority) to refresh it - no program redeploy needed.
const URI = "";

// Metaplex Token Metadata program - same address on every cluster.
const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

async function main() {
  const secret = process.env.DEPLOYER_PRIVATE_KEY;
  if (!secret) throw new Error("DEPLOYER_PRIVATE_KEY missing from .env");

  const admin = Keypair.fromSecretKey(bs58.decode(secret));
  const connection = new Connection(
    process.env.RPC_URL || "https://api.devnet.solana.com",
    "confirmed"
  );
  const wallet = new anchor.Wallet(admin);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const program = new anchor.Program(
    idl as anchor.Idl,
    provider
  ) as unknown as anchor.Program<SplTokenDeploy>;

  const mint = Keypair.generate();
  const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint_authority"), mint.publicKey.toBuffer()],
    program.programId
  );
  const [metadata] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
    TOKEN_METADATA_PROGRAM_ID
  );

  console.log("Admin:", admin.publicKey.toBase58());
  console.log("New mint:", mint.publicKey.toBase58());
  console.log("Mint authority PDA:", mintAuthority.toBase58());
  console.log(`Name / Symbol: "${NAME}" / "${SYMBOL}"`);

  // .accountsPartial() (not .accounts()) - Anchor 0.31's stricter
  // `.accounts()` type tries to auto-resolve every PDA/relation-tagged
  // account and rejects you supplying them yourself, even when - like here -
  // you already computed the exact right address. accountsPartial() accepts
  // any subset you hand it and lets Anchor fill in only what's missing.
  const sig = await program.methods
    .createToken(DECIMALS, NAME, SYMBOL, URI)
    .accountsPartial({
      admin: admin.publicKey,
      mint: mint.publicKey,
      mintAuthority,
      metadata,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([mint])
    .rpc();

  console.log("create_token tx:", sig);

  const outPath = path.join(__dirname, "..", "deployed-mint.json");
  fs.writeFileSync(
    outPath,
    JSON.stringify(
      { mint: mint.publicKey.toBase58(), decimals: DECIMALS, name: NAME, symbol: SYMBOL },
      null,
      2
    )
  );
  console.log(`Saved mint address to ${outPath}`);
  console.log(
    `\nView it on Solana Explorer:\n` +
      `https://explorer.solana.com/address/${mint.publicKey.toBase58()}?cluster=devnet`
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
