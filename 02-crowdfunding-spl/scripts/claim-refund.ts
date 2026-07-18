// Contributor claims their money back after a campaign expired without
// hitting its goal. Bonus instruction not present in the reference
// Solidity contract - see README "Teaching notes".
import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import idl from "../target/idl/crowdfunding.json";
import type { Crowdfunding } from "../target/types/crowdfunding";

async function main() {
  const secret = process.env.DEPLOYER_PRIVATE_KEY;
  if (!secret) throw new Error("DEPLOYER_PRIVATE_KEY missing from .env");

  const contributor = Keypair.fromSecretKey(bs58.decode(secret));
  const connection = new Connection(process.env.RPC_URL || "https://api.devnet.solana.com", "confirmed");
  const wallet = new anchor.Wallet(contributor);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);
  const program = new anchor.Program(idl as anchor.Idl, provider) as unknown as anchor.Program<Crowdfunding>;

  const campaignFile = path.join(__dirname, "..", "last-campaign.json");
  if (!fs.existsSync(campaignFile)) throw new Error("Run `yarn create-campaign` first");
  const { campaign: campaignStr } = JSON.parse(fs.readFileSync(campaignFile, "utf-8"));
  const campaign = new PublicKey(campaignStr);

  const campaignAccount = await program.account.campaign.fetch(campaign);
  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), campaign.toBuffer()],
    program.programId
  );
  const [contribution] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("contribution"), campaign.toBuffer(), contributor.publicKey.toBuffer()],
    program.programId
  );
  const contributorTokenAccount = getAssociatedTokenAddressSync(
    campaignAccount.tokenMint,
    contributor.publicKey
  );

  const sig = await program.methods
    .claimRefund()
    .accounts({
      contributor: contributor.publicKey,
      campaign,
      vault,
      contribution,
      contributorTokenAccount,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("claim_refund tx:", sig);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
