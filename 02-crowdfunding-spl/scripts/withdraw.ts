// Creator withdraws raised funds once the goal is met. Must be run by the
// same wallet that created the campaign.
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

  const creator = Keypair.fromSecretKey(bs58.decode(secret));
  const connection = new Connection(process.env.RPC_URL || "https://api.devnet.solana.com", "confirmed");
  const wallet = new anchor.Wallet(creator);
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
  const creatorTokenAccount = getAssociatedTokenAddressSync(
    campaignAccount.tokenMint,
    creator.publicKey
  );

  const sig = await program.methods
    .withdraw()
    .accounts({
      creator: creator.publicKey,
      campaign,
      vault,
      tokenMint: campaignAccount.tokenMint,
      creatorTokenAccount,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .rpc();

  console.log("withdraw tx:", sig);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
