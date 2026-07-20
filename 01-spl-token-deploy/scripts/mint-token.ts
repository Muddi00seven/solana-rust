// Mints tokens from the mint created by create-token.ts to a recipient
// wallet's associated token account. Signs with DEPLOYER_PRIVATE_KEY (must
// be the same admin that created the token).
//
// Usage:
//   yarn mint                                  (mints to yourself)
//   yarn mint -- <RECIPIENT_PUBKEY> <AMOUNT>    (mints to someone else)
import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import idl from "../target/idl/spl_token_deploy.json";
import type { SplTokenDeploy } from "../target/types/spl_token_deploy";

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

  const mintFile = path.join(__dirname, "..", "deployed-mint.json");
  if (!fs.existsSync(mintFile)) {
    throw new Error("deployed-mint.json not found - run `yarn create-token` first");
  }
  const { mint, decimals } = JSON.parse(fs.readFileSync(mintFile, "utf-8"));
  const mintPubkey = new PublicKey(mint);

  const args = process.argv.slice(2);
  const recipient = args[0] ? new PublicKey(args[0]) : admin.publicKey;
  const humanAmount = args[1] ? Number(args[1]) : 1000;
  const rawAmount = new anchor.BN(humanAmount).mul(new anchor.BN(10).pow(new anchor.BN(decimals)));

  const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint_authority"), mintPubkey.toBuffer()],
    program.programId
  );
  const recipientTokenAccount = getAssociatedTokenAddressSync(mintPubkey, recipient);

  console.log(`Minting ${humanAmount} tokens to ${recipient.toBase58()}`);

  // .accountsPartial() - see the comment in create-token.ts for why this is
  // used instead of .accounts() with this Anchor version.
  const sig = await program.methods
    .mintToWallet(rawAmount)
    .accountsPartial({
      admin: admin.publicKey,
      mint: mintPubkey,
      mintAuthority,
      recipient,
      recipientTokenAccount,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .rpc();

  console.log("mint_to_wallet tx:", sig);
  console.log(`Token account: ${recipientTokenAccount.toBase58()}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
