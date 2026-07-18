// Local-validator test - mints an NFT with a fake (non-IPFS) URI so this
// can run fully offline via `anchor test`. The real IPFS flow is exercised
// by scripts/upload-to-ipfs.ts + scripts/mint-nft.ts against devnet.
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, getAccount } from "@solana/spl-token";
import { assert } from "chai";
import { NftMint } from "../target/types/nft_mint";

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("nft_mint", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.NftMint as Program<NftMint>;
  const payer = provider.wallet as anchor.Wallet;

  it("mints a 1-of-1 NFT with metadata + master edition", async () => {
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

    await program.methods
      .mintNft("Test NFT", "TEST", "https://example.com/metadata.json")
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

    const tokenAccount = await getAccount(provider.connection, ownerTokenAccount);
    assert.equal(tokenAccount.amount.toString(), "1");

    const mintInfo = await provider.connection.getParsedAccountInfo(mint.publicKey);
    // @ts-ignore - parsed shape
    const parsed = mintInfo.value?.data?.parsed?.info;
    assert.equal(parsed.decimals, 0);
    assert.equal(parsed.supply, "1");
  });
});
