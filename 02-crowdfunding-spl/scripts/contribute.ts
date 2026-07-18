// Contributes tokens to the last campaign created by create-campaign.ts.
// Usage: yarn contribute -- <humanAmount>
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

  const humanAmount = Number(process.argv[2] || "10");
  const mintInfo = await connection.getParsedAccountInfo(campaignAccount.tokenMint);
  // @ts-ignore
  const decimals = mintInfo.value?.data?.parsed?.info?.decimals ?? 6;
  const amount = new anchor.BN(humanAmount).mul(new anchor.BN(10).pow(new anchor.BN(decimals)));

  const sig = await program.methods
    .contribute(amount)
    .accounts({
      contributor: contributor.publicKey,
      campaign,
      vault,
      contribution,
      contributorTokenAccount,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("contribute tx:", sig);
  const updated = await program.account.campaign.fetch(campaign);
  console.log(`Total raised: ${updated.raised.toString()} / goal ${updated.goal.toString()}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
