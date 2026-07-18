// Creates a new crowdfunding pool. Usage:
//   yarn create-campaign -- "Title here" "Description" <goalHumanAmount> <durationDays> [imageUrl]
import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import idl from "../target/idl/crowdfunding.json";
import type { Crowdfunding } from "../target/types/crowdfunding";

async function main() {
  const secret = process.env.DEPLOYER_PRIVATE_KEY;
  if (!secret) throw new Error("DEPLOYER_PRIVATE_KEY missing from .env");
  const tokenMintStr = process.env.TOKEN_MINT;
  if (!tokenMintStr) throw new Error("TOKEN_MINT missing from .env - run `yarn setup-test-mint` first");

  const creator = Keypair.fromSecretKey(bs58.decode(secret));
  const connection = new Connection(process.env.RPC_URL || "https://api.devnet.solana.com", "confirmed");
  const wallet = new anchor.Wallet(creator);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const program = new anchor.Program(idl as anchor.Idl, provider) as unknown as anchor.Program<Crowdfunding>;
  const tokenMint = new PublicKey(tokenMintStr);

  const [title, description, goalStr, durationStr, imageUrl] = process.argv.slice(2);
  if (!title || !goalStr || !durationStr) {
    console.error(
      'Usage: yarn create-campaign -- "Title" "Description" <goal> <durationDays> [imageUrl]'
    );
    process.exit(1);
  }

  const mintInfo = await connection.getParsedAccountInfo(tokenMint);
  // @ts-ignore
  const decimals = mintInfo.value?.data?.parsed?.info?.decimals ?? 6;
  const goal = new anchor.BN(Number(goalStr)).mul(new anchor.BN(10).pow(new anchor.BN(decimals)));

  // campaign_id: a simple per-creator nonce. Using the current timestamp
  // keeps this demo script simple; a real app would track a counter.
  const campaignId = new anchor.BN(Date.now());

  const [campaign] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("campaign"), creator.publicKey.toBuffer(), campaignId.toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), campaign.toBuffer()],
    program.programId
  );

  const sig = await program.methods
    .createCampaign(
      campaignId,
      title,
      description || "",
      imageUrl || "",
      goal,
      new anchor.BN(Number(durationStr))
    )
    .accounts({
      creator: creator.publicKey,
      tokenMint,
      campaign,
      vault,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .rpc();

  console.log("create_campaign tx:", sig);
  console.log("Campaign PDA:", campaign.toBase58());
  console.log("Vault:", vault.toBase58());

  fs.writeFileSync(
    path.join(__dirname, "..", "last-campaign.json"),
    JSON.stringify({ campaign: campaign.toBase58(), campaignId: campaignId.toString() }, null, 2)
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
