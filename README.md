# Solana + Rust, via Anchor — 3 Lecture Projects

Three self-contained Anchor workspaces, one per lecture, for teaching
Solana development to a small cohort (<10 students) who already know
another language but are new to Rust. Each folder builds and deploys
independently.

| # | Folder | What it teaches |
|---|--------|------------------|
| 1 | [`01-spl-token-deploy/`](./01-spl-token-deploy) | Installing the toolchain from scratch, Anchor project anatomy, PDAs as mint authorities, deploying/minting a custom SPL token |
| 2 | [`02-crowdfunding-spl/`](./02-crowdfunding-spl) | Porting a real EVM contract (see below) to Solana - one PDA per campaign/contribution, vaults, CPI token transfers, a PDA signing for itself |
| 3 | [`03-nft-ipfs/`](./03-nft-ipfs) | CPI into a *foreign* program (Metaplex Token Metadata), what actually makes an SPL token an "NFT," pinning metadata to IPFS |

Suggested pacing: one folder per class session (~2-3 hours each), with
students doing the install (project 1, section 1) as homework before
session 1 so lecture time goes to concepts, not `brew install` waiting.

## Before class: assign this as pre-work

Rust Book chapters 1-6 (ownership, structs, enums, `Result`/`Option`) or
the Rustlings exercises. Anchor handles most of Rust's boilerplate for you,
so deep ownership mastery isn't required - just enough syntax comfort that
lecture time isn't spent explaining `struct` and `impl`.

## Project 2's reference contract

Project 2 ports the logic of an existing Solidity crowdfunding dapp
("ChainFund," in the sibling `crowd-funding-dapp` project this repo was
built alongside) - same Campaign/Contribution model, same validation
rules, but rebuilt around Solana's account model instead of contract
storage. Comparing the two side by side is the fastest way to make PDAs
and CPIs click for students coming from EVM.

## Verification status (read this before you start teaching)

All three programs were actually compiled end-to-end on a real Mac -
`cargo check` (fast type/logic check) and a full `anchor build` producing
real `.so` binaries - not just written and assumed to work. Three genuine
Anchor bugs were caught and fixed in the process:

1. **Constraint ordering** (`01-spl-token-deploy`): a `has_one`/`address`
   constraint referencing another account only compiles if that account is
   declared *earlier* in the `#[derive(Accounts)]` struct - Anchor
   validates fields top-to-bottom. This is the single most common Anchor
   compile error and shows up twice across these three programs; worth its
   own five minutes in lecture 1.
2. **`init_if_needed` needs an explicit feature flag** (`anchor-lang =
   { features = ["init-if-needed"] }` in `Cargo.toml`) - easy to forget,
   the compiler error tells you exactly what's missing.
3. **`associated_token::mint = ...` on an `init` account needs a real
   `Account<Mint>` field**, not just a `Pubkey` expression like
   `campaign.token_mint` (`02-crowdfunding-spl`, in `Withdraw`).

Each project's own README has the specifics for its bugs and a "Teaching
notes" section pointing out where to slow down.

### The toolchain gotcha every student will hit

`anchor build` compiles with a Rust toolchain *bundled inside the Solana
CLI* (`cargo-build-sbf`), separate from your system Rust. It lags behind
crates.io, so a handful of common transitive dependencies now published
under newer editions/MSRVs (`blake3`, `zeroize_derive`, `indexmap`,
`jobserver`, `unicode-segmentation`, the `toml_edit`/`proc-macro-crate`
chain) will fail to build with errors like:

```
error: failed to parse manifest ... feature `edition2024` is required
error: rustc 1.79.0-dev is not supported by the following packages: ...
```

Every `Cargo.lock` in this repo already has these pinned to compatible
versions - **commit `Cargo.lock`, don't gitignore it**, which is why it
isn't in any of the `.gitignore` files here. If a student deletes/
regenerates their lock file and hits this, the fix is always the same:
read cargo's own error message (it names the exact package) and run
```
cargo update -p <package> --precise <older-version>
```
working from the outermost failure inward. There's also a known
`anchor-lang 0.30.1` bug where the *IDL generation* step (not the program
build itself) fails with `no method named 'source_file' found for struct
'proc_macro2::Span'` - work around it with `anchor build --no-idl`; the
deployable program is unaffected.

## Repo layout

```
solana-rust/
  01-spl-token-deploy/   independent Anchor workspace
  02-crowdfunding-spl/   independent Anchor workspace
  03-nft-ipfs/           independent Anchor workspace
```

No shared root `package.json`/`Cargo.toml` - each folder is deliberately
standalone so students (or you) can `cd` into just one and follow that
project's README without the other two being relevant.
