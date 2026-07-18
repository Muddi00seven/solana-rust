// Only needed if you didn't set TOKEN_MINT in .env: creates a throwaway SPL
// token (6 decimals) and mints 10,000 of it to the deployer wallet, purely
// so there's something to run campaigns with in class. In the real world
// you'd point TOKEN_MINT at an existing token (USDC, project 1's token,
// etc).
import "dotenv/config";
import { Keypair, Connection } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import bs58 from "bs58";
import fs from "fs";
import path from "path";

async function main() {
  const secret = process.env.DEPLOYER_PRIVATE_KEY;
  if (!secret) throw new Error("DEPLOYER_PRIVATE_KEY missing from .env");

  const payer = Keypair.fromSecretKey(bs58.decode(secret));
  const connection = new Connection(
    process.env.RPC_URL || "https://api.devnet.solana.com",
    "confirmed"
  );

  console.log("Creating test mint (6 decimals)...");
  const mint = await createMint(connection, payer, payer.publicKey, payer.publicKey, 6);
  console.log("Test mint:", mint.toBase58());

  const ata = await getOrCreateAssociatedTokenAccount(connection, payer, mint, payer.publicKey);
  await mintTo(connection, payer, mint, ata.address, payer, 10_000 * 10 ** 6);
  console.log(`Minted 10,000 test tokens to ${ata.address.toBase58()}`);

  const envPath = path.join(__dirname, "..", ".env");
  const env = fs.readFileSync(envPath, "utf-8");
  const updated = env.includes("TOKEN_MINT=")
    ? env.replace(/TOKEN_MINT=.*/, `TOKEN_MINT=${mint.toBase58()}`)
    : `${env}\nTOKEN_MINT=${mint.toBase58()}\n`;
  fs.writeFileSync(envPath, updated);
  console.log("Wrote TOKEN_MINT into .env");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
