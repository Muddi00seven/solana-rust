import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";
import { Crowdfunding } from "../target/types/crowdfunding";

describe("crowdfunding", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Crowdfunding as Program<Crowdfunding>;
  const creator = provider.wallet as anchor.Wallet;

  const decimals = 6;
  const goalHuman = 100;
  const goal = new anchor.BN(goalHuman).mul(new anchor.BN(10).pow(new anchor.BN(decimals)));
  const campaignId = new anchor.BN(Date.now());

  let tokenMint: PublicKey;
  let campaign: PublicKey;
  let vault: PublicKey;
  let creatorTokenAccount: PublicKey;

  before(async () => {
    tokenMint = await createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      creator.publicKey,
      creator.publicKey,
      decimals
    );
    const ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      tokenMint,
      creator.publicKey
    );
    creatorTokenAccount = ata.address;
    // Mint enough to fully fund the campaign's goal from this same wallet -
    // fine for a test, real campaigns obviously have many contributors.
    await mintTo(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      tokenMint,
      creatorTokenAccount,
      creator.publicKey,
      goalHuman * 10 ** decimals
    );

    [campaign] = PublicKey.findProgramAddressSync(
      [Buffer.from("campaign"), creator.publicKey.toBuffer(), campaignId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), campaign.toBuffer()],
      program.programId
    );
  });

  it("creates a campaign", async () => {
    await program.methods
      .createCampaign(campaignId, "Solana Class Fund", "Raised in lecture 2", "", goal, new anchor.BN(30))
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

    const account = await program.account.campaign.fetch(campaign);
    assert.equal(account.creator.toBase58(), creator.publicKey.toBase58());
    assert.equal(account.goal.toString(), goal.toString());
    assert.equal(account.raised.toString(), "0");
    assert.equal(account.withdrawn, false);
  });

  it("accepts a contribution and updates raised/contributors_count", async () => {
    const [contribution] = PublicKey.findProgramAddressSync(
      [Buffer.from("contribution"), campaign.toBuffer(), creator.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .contribute(goal) // fund the whole goal in one go so withdraw can be tested
      .accounts({
        contributor: creator.publicKey,
        campaign,
        vault,
        contribution,
        contributorTokenAccount: creatorTokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.campaign.fetch(campaign);
    assert.equal(account.raised.toString(), goal.toString());
    assert.equal(account.contributorsCount.toString(), "1");

    const vaultAccount = await getAccount(provider.connection, vault);
    assert.equal(vaultAccount.amount.toString(), goal.toString());
  });

  it("lets the creator withdraw once the goal is met", async () => {
    await program.methods
      .withdraw()
      .accounts({
        creator: creator.publicKey,
        campaign,
        vault,
        tokenMint,
        creatorTokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const account = await program.account.campaign.fetch(campaign);
    assert.equal(account.withdrawn, true);
  });

  it("rejects a second withdraw", async () => {
    try {
      await program.methods
        .withdraw()
        .accounts({
          creator: creator.publicKey,
          campaign,
          vault,
          tokenMint,
          creatorTokenAccount,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();
      assert.fail("expected second withdraw to be rejected");
    } catch (err) {
      assert.include(String(err), "AlreadyWithdrawn");
    }
  });

  // claim_refund needs a campaign whose deadline has already passed, which
  // means warping the local validator's clock (e.g. with solana-bankrun or
  // `solana-test-validator --warp-slot`) - a good follow-up exercise for
  // students rather than something to hardcode into a basic lecture test.
});
