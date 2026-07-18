// Calls the deployed nft_mint program's mint_nft instruction using the
// name/symbol/uri saved by scripts/upload-to-ipfs.ts. Signs with
// DEPLOYER_PRIVATE_KEY from .env - this wallet becomes the NFT's owner AND
// (until master edition creation locks it down) its mint/update authority.
import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import idl from "../target/idl/nft_mint.json";
import type { NftMint } from "../target/types/nft_mint";

// Metaplex Token Metadata program - same address on every cluster.
const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

async function main() {
  const secret = process.env.DEPLOYER_PRIVATE_KEY;
  if (!secret) throw new Error("DEPLOYER_PRIVATE_KEY missing from .env");

  const payer = Keypair.fromSecretKey(bs58.decode(secret));
  const connection = new Connection(
    process.env.RPC_URL || "https://api.devnet.solana.com",
    "confirmed"
  );
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const program = new anchor.Program(
    idl as anchor.Idl,
    provider
  ) as unknown as anchor.Program<NftMint>;

  const pendingPath = path.join(__dirname, "..", "pending-mint.json");
  if (!fs.existsSync(pendingPath)) {
    throw new Error("pending-mint.json not found - run `yarn upload` first");
  }
  const { name, symbol, uri } = JSON.parse(fs.readFileSync(pendingPath, "utf-8"));

  const mint = Keypair.generate();
  const ownerTokenAccount = getAssociatedTokenAddressSync(mint.publicKey, payer.publicKey);

  const [metadata] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
    TOKEN_METADATA_PROGRAM_ID
  );
  const [masterEdition] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.publicKey.toBuffer(),
      Buffer.from("edition"),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

  console.log(`Minting "${name}" (${symbol})`);
  console.log(`Mint: ${mint.publicKey.toBase58()}`);
  console.log(`Metadata URI: ${uri}`);

  const sig = await program.methods
    .mintNft(name, symbol, uri)
    .accounts({
      payer: payer.publicKey,
      mint: mint.publicKey,
      ownerTokenAccount,
      metadata,
      masterEdition,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([mint])
    .rpc();

  console.log("mint_nft tx:", sig);
  console.log(
    `\nView it:\n` +
      `  https://explorer.solana.com/address/${mint.publicKey.toBase58()}?cluster=devnet\n` +
      `  Phantom/Solflare devnet wallet (add the devnet RPC) should show it too.`
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
