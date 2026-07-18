# Project 1 — Install Everything + Deploy Your Own SPL Token via Anchor

Lecture 1 goal: go from a blank machine to a custom SPL token minted on
devnet, using a real Anchor program (not just the `spl-token` CLI) so you
can see how mint authority, PDAs, and CPIs work under the hood.

## 0. What you're building

An Anchor program with two instructions:

- `create_token` — creates a new SPL Mint account. The mint authority is a
  PDA (`mint_authority`) owned by the program, not a wallet.
- `mint_to_wallet` — mints more tokens to any wallet's associated token
  account, but only the original `admin` (stored on the PDA) is allowed to
  call it.

This is the difference between "just run a CLI command" and "deploy a
program that controls token issuance with your own rules."

## 1. Install the toolchain

Do this once per machine.

### Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version   # sanity check
```

### Solana CLI
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version
```
(Anza is the maintainer of the Solana CLI/validator now — this is the
current official install command as of 2026. If it 404s, check
https://docs.anza.xyz/cli/install for the latest one-liner.)

### Node.js + Yarn
Install Node 20 LTS from [nodejs.org](https://nodejs.org), then:
```bash
npm install -g yarn
```

### Anchor (via AVM — the Anchor Version Manager)
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
anchor --version
```

## 2. Create + fund a devnet wallet

```bash
solana config set --url devnet
solana-keygen new --outfile ~/deployer-raw.json
solana airdrop 2 $(solana-keygen pubkey ~/deployer-raw.json) --url devnet
```

If the airdrop is rate-limited, use https://faucet.solana.com instead.

## 3. Put the private key in `.env`

This project never asks for your key in chat/plaintext scripts you didn't
write yourself — you convert your own keypair file to base58 locally:

```bash
cp .env.example .env
node scripts/keypair-to-base58.js ~/deployer-raw.json
# paste the printed string into .env as DEPLOYER_PRIVATE_KEY
```

Then materialize it as the Anchor CLI wallet file:
```bash
yarn install
yarn wallet:from-env
```
This writes `.wallet/deployer.json` (used by `Anchor.toml`'s
`[provider].wallet`) from the same key in `.env` — one source of truth.

## 4. Build and deploy

```bash
anchor keys list                # note the generated program id
```
Copy that id into `declare_id!()` in `programs/spl_token_deploy/src/lib.rs`
and into `Anchor.toml` under `[programs.localnet]` / `[programs.devnet]`
(or just run `anchor keys sync`).

```bash
anchor build
anchor deploy --provider.cluster devnet
```

## 5. Run the local test suite first (recommended)

Before touching devnet, sanity check the program logic on a local
validator Anchor spins up for you:
```bash
anchor test
```
This runs `tests/spl_token_deploy.ts`: creates a token, mints to yourself,
and asserts a non-admin wallet gets rejected.

## 6. Create and mint your token on devnet

```bash
yarn create-token      # creates the mint, saves deployed-mint.json
yarn mint               # mints 1000 tokens to your own wallet
yarn mint -- <SOME_OTHER_WALLET> 250   # mint 250 to someone else
```

## 7. Verify

```bash
solana address -k .wallet/deployer.json
spl-token accounts --owner <your-pubkey> --url devnet
```
Or just paste the mint address from `deployed-mint.json` into
https://explorer.solana.com/?cluster=devnet.

## Teaching notes

- Walk through **why** the mint authority is a PDA and not the admin's own
  wallet: it means minting rules live in program code, auditable and
  enforceable on-chain, instead of "trust me, I won't mint more."
- `has_one` vs `address` constraints: this program had a real ordering bug
  during writing — Anchor validates `#[account(...)]` constraints in the
  field-declaration order, so a constraint referencing another field only
  works if that field is declared *earlier* in the struct. Good live demo
  of reading Anchor macro-expansion errors.
- `mint::authority = mint_authority` / `mint::freeze_authority =
  mint_authority` in the `create_token` accounts struct is Anchor's
  shorthand for "run the CPI to the Token Program that sets these fields
  on init" — show students the expanded CPI Anchor generates for you if
  time allows (`anchor expand`).

## Verification status

This program was actually built end to end (`cargo check` + full `anchor
build`, producing a real `spl_token_deploy.so`) against Anchor CLI
`0.30.1` / Solana CLI `2.1.16` on a real Mac, not just written and hoped
for. Two real bugs were caught and fixed in the process: a `has_one`
constraint referencing an account declared *after* it in the struct
(Anchor validates fields top-to-bottom, so cross-references only work
pointing at earlier fields), and `init_if_needed` needing the
`init-if-needed` Anchor feature explicitly enabled in `Cargo.toml`.

### A toolchain gotcha you'll likely hit too

`anchor build` uses `cargo-build-sbf`, which bundles its **own** older
Rust compiler separate from your system Rust. Several transitive
dependencies (`blake3`, `zeroize_derive`, `indexmap`, `jobserver`,
`unicode-segmentation`, the `toml_edit`/`proc-macro-crate` chain) have
since published newer versions that need a newer compiler than that
bundled one, causing errors like `feature 'edition2024' is required` or
`rustc 1.79.0-dev is not supported`. `Cargo.lock` in this project already
has all of those pinned down to compatible versions (via `cargo update -p
<name> --precise <version>`) — if you ever delete/regenerate the lock file
and hit this again, cargo's own error message tells you exactly which
package to pin.

If `anchor build` fails specifically at the IDL-generation step with `no
method named 'source_file' found for struct 'proc_macro2::Span'`, that's a
known `anchor-lang 0.30.1` bug against newer `proc_macro2` — build with
`anchor build --no-idl` instead; the on-chain program is unaffected, you
just skip the client-side TypeScript IDL JSON.
