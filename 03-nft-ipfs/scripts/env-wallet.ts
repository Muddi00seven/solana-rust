// Same pattern as projects 1 and 2 - materializes DEPLOYER_PRIVATE_KEY from
// .env into .wallet/deployer.json for the Anchor CLI.
import "dotenv/config";
import fs from "fs";
import path from "path";
import bs58 from "bs58";
import { Keypair, Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";

async function main() {
  const secret = process.env.DEPLOYER_PRIVATE_KEY;
  if (!secret) {
    console.error("DEPLOYER_PRIVATE_KEY is not set in .env - copy .env.example first.");
    process.exit(1);
  }

  const keypair = Keypair.fromSecretKey(bs58.decode(secret));
  const walletDir = path.join(__dirname, "..", ".wallet");
  fs.mkdirSync(walletDir, { recursive: true });
  const walletPath = path.join(walletDir, "deployer.json");
  fs.writeFileSync(walletPath, JSON.stringify(Array.from(keypair.secretKey)));

  console.log(`Wrote keypair file: ${walletPath}`);
  console.log(`Public key: ${keypair.publicKey.toBase58()}`);

  const rpcUrl = process.env.RPC_URL || "https://api.devnet.solana.com";
  const connection = new Connection(rpcUrl, "confirmed");
  const balance = await connection.getBalance(keypair.publicKey);
  console.log(`Devnet balance: ${balance / LAMPORTS_PER_SOL} SOL`);
  if (balance === 0) {
    console.log(`\nAirdrop: solana airdrop 2 ${keypair.publicKey.toBase58()} --url devnet`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
