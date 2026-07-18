# Project 3 — Mint an NFT with Anchor + Metaplex + IPFS

Lecture 3 goal: mint a real, wallet-visible NFT (shows up in Phantom/
Solflare, not just a raw SPL token) by combining everything from lectures
1-2 with one new idea - CPI-ing into a *foreign* program (Metaplex's Token
Metadata program) instead of just the Token Program.

## 0. What makes something an "NFT" on Solana

There's no separate "NFT program." An NFT is just:
1. An SPL Mint with **0 decimals** and **supply capped at 1**.
2. A **Metadata account** (owned by Metaplex's Token Metadata program,
   address derived from `["metadata", token_metadata_program, mint]`)
   holding the name/symbol/uri.
3. A **Master Edition account** (same program, seeds + `"edition"`) that
   locks the mint authority so nobody can mint a 2nd copy.

`programs/nft_mint/src/lib.rs`'s single instruction, `mint_nft`, does all
three steps in one transaction.

## 1. Prerequisites

Same toolchain as projects 1 and 2 (Rust, Solana CLI, Anchor via avm, Node
+ Yarn) - see `../01-spl-token-deploy/README.md` section 1 if you haven't
installed them yet. This project additionally needs a free
[Pinata](https://pinata.cloud) account for IPFS uploads.

## 2. Wallet setup

```bash
cp .env.example .env
node ../01-spl-token-deploy/scripts/keypair-to-base58.js ~/deployer-raw.json
# paste into .env as DEPLOYER_PRIVATE_KEY (same wallet as before is fine)
yarn install
yarn wallet:from-env
```

Get a Pinata JWT: dashboard -> API Keys -> New Key -> enable
`pinFileToIPFS` + `pinJSONToIPFS` -> copy the **JWT** (not the key/secret
pair) into `.env` as `PINATA_JWT`.

## 3. Build and deploy

```bash
anchor keys list        # note the program id
# paste it into declare_id!() in programs/nft_mint/src/lib.rs and Anchor.toml
# (or run `anchor keys sync`)
anchor build
anchor deploy --provider.cluster devnet
```

## 4. Test locally first

```bash
anchor test
```
`Anchor.toml` has a `[[test.validator.clone]]` entry that pulls the real
Metaplex Token Metadata program onto the local validator - without it,
`anchor test`'s CPI into Metaplex would fail with "program not found."
Worth pointing out to students: this is the standard pattern for testing
against *any* program you don't own.

## 5. Upload to IPFS, then mint on devnet

```bash
mkdir -p assets && cp /path/to/your/image.png assets/image.png
yarn upload "My NFT" "SYM" "A description of it"
yarn mint
```

`yarn upload` pins the image, then a Metaplex-standard metadata JSON
(`{name, symbol, description, image, attributes, properties}`) to IPFS via
Pinata, and saves the resulting `ipfs://` URI to `pending-mint.json`.
`yarn mint` reads that file and calls `mint_nft` on-chain.

## 6. Verify

Paste the mint address (printed by `yarn mint`) into
https://explorer.solana.com/?cluster=devnet - you should see Metadata and
Edition accounts alongside the Mint. Add the devnet RPC to Phantom/Solflare
and the NFT should render there too (pulling the image straight from IPFS
through your metadata URI).

## Teaching notes

- Compare this to project 1's `mint_authority` PDA: here the **payer's own
  wallet** is the mint/update authority, on purpose - there's no reason to
  add PDA complexity for a one-shot NFT mint. Good moment to ask "when
  would you *want* a PDA authority instead?" (answer: anything the program
  needs to control repeatedly, like project 1's re-mintable token or
  project 2's vault).
- `seeds::program = token_metadata_program.key()` on the `metadata` /
  `master_edition` accounts is new syntax vs. projects 1-2 - it tells
  Anchor "derive this PDA against a *different* program's ID, not our
  own," which is exactly what CPI-ing into someone else's program requires.
- `UncheckedAccount` + `/// CHECK:` comments: these accounts aren't
  deserialized by our program at all - Metaplex's own program validates
  them when we CPI in. This is the right pattern whenever you're handing
  an account to another program to manage.
- `max_supply: Some(0)` in `create_master_edition_v3` is what actually
  prevents anyone (including the original authority) from minting a 2nd
  copy - worth contrasting with `None` (unlimited prints, used for things
  like concert tickets sold as multiple copies of one "edition").

## If `anchor build` fails

Like projects 1-2, this was hand-written and reviewed but not compiled in
the sandbox that generated this repo (no crates.io access there). The
`anchor-spl` `metadata` feature flag (already set in
`programs/nft_mint/Cargo.toml`) is what pulls in the Metaplex CPI helpers -
if it doesn't build, first check that feature is still there, and that
your installed Anchor CLI version matches the `0.30.1` pinned in
`Cargo.toml`.

### A real toolchain gotcha you will probably also hit

While verifying this repo we hit `anchor build` failing with errors like
`feature 'edition2024' is required` or `rustc 1.79.0-dev is not supported
by ... requires rustc 1.8x`. This happens because `cargo-build-sbf` (bundled
with the Solana CLI) ships its **own** older Rust compiler, separate from
your system Rust - and some transitive dependencies (`blake3`,
`zeroize_derive`, `indexmap`, `jobserver`, `unicode-segmentation`, the
`toml_edit`/`proc-macro-crate` chain) have since published newer versions
that need a newer compiler than whatever your Solana CLI bundles.

Fix: pin the offending crate(s) down with `cargo update -p <name> --precise
<older-version>` (cargo's error message tells you exactly which package and
usually which version range to aim for). If `anchor build` also fails
specifically at the *IDL generation* step with `no method named
'source_file' found for struct 'proc_macro2::Span'`, that's a separate
known bug in `anchor-lang 0.30.1`'s IDL builder against newer
`proc_macro2` releases - just build with `anchor build --no-idl` (the
on-chain program itself is unaffected; you're only skipping the
client-side TypeScript IDL JSON, which you can regenerate later once
anchor-lang patches it).
