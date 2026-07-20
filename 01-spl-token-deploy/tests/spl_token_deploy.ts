// Runs against `anchor test`'s local validator (not devnet) - fast feedback
// loop while writing/teaching the program, separate from the devnet
// scripts/*.ts flow used for the real deploy.
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, getAccount } from "@solana/spl-token";
import { assert } from "chai";
import { SplTokenDeploy } from "../target/types/spl_token_deploy";

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("spl_token_deploy", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SplTokenDeploy as Program<SplTokenDeploy>;

  const admin = provider.wallet as anchor.Wallet;
  const mint = Keypair.generate();
  const decimals = 6;

  let mintAuthority: PublicKey;

  it("creates a new token", async () => {
    [mintAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority"), mint.publicKey.toBuffer()],
      program.programId
    );
    const [metadata] = PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID
    );

    await program.methods
      .createToken(decimals, "Test Token", "TST", "")
      .accountsPartial({
        admin: admin.publicKey,
        mint: mint.publicKey,
        mintAuthority,
        metadata,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([mint])
      .rpc();

    const authorityAccount = await program.account.mintAuthority.fetch(mintAuthority);
    assert.equal(authorityAccount.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(authorityAccount.mint.toBase58(), mint.publicKey.toBase58());
  });

  it("mints tokens to the admin's own wallet", async () => {
    const recipientTokenAccount = getAssociatedTokenAddressSync(
      mint.publicKey,
      admin.publicKey
    );
    const amount = new anchor.BN(1_000).mul(new anchor.BN(10).pow(new anchor.BN(decimals)));

    await program.methods
      .mintToWallet(amount)
      .accountsPartial({
        admin: admin.publicKey,
        mint: mint.publicKey,
        mintAuthority,
        recipient: admin.publicKey,
        recipientTokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const account = await getAccount(provider.connection, recipientTokenAccount);
    assert.equal(account.amount.toString(), amount.toString());
  });

  it("rejects mint_to_wallet from a non-admin signer", async () => {
    const impostor = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(impostor.publicKey, 1e9);
    await provider.connection.confirmTransaction(sig);

    const recipientTokenAccount = getAssociatedTokenAddressSync(
      mint.publicKey,
      impostor.publicKey
    );

    try {
      await program.methods
        .mintToWallet(new anchor.BN(1))
        .accountsPartial({
          admin: impostor.publicKey,
          mint: mint.publicKey,
          mintAuthority,
          recipient: impostor.publicKey,
          recipientTokenAccount,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([impostor])
        .rpc();
      assert.fail("expected mint_to_wallet to reject a non-admin signer");
    } catch (err) {
      assert.include(String(err), "NotAdmin");
    }
  });
});
