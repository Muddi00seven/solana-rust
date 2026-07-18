// Calls the deployed spl_token_deploy program's create_token instruction to
// mint a brand new SPL token. Signs with DEPLOYER_PRIVATE_KEY from .env.
//
// Usage: yarn create-token   (after `anchor deploy`)
import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection } from "@solana/web3.js";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import idl from "../target/idl/spl_token_deploy.json";
import type { SplTokenDeploy } from "../target/types/spl_token_deploy";

const DECIMALS = 6; // change if you want a different precision

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

  console.log("Admin:", admin.publicKey.toBase58());
  console.log("New mint:", mint.publicKey.toBase58());
  console.log("Mint authority PDA:", mintAuthority.toBase58());

  const sig = await program.methods
    .createToken(DECIMALS)
    .accounts({
      admin: admin.publicKey,
      mint: mint.publicKey,
      mintAuthority,
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
    JSON.stringify({ mint: mint.publicKey.toBase58(), decimals: DECIMALS }, null, 2)
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
