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

## Installation Guide (do this once per machine)

You need four things, in this order: **Node.js**, **Rust**, the **Solana
CLI**, and **Anchor**. Each depends on the previous one being on your
`PATH`, so install top to bottom and open a fresh terminal after each step
if a command "isn't found."

### Windows

Solana's build tooling (`cargo-build-sbf`) is Linux-first and isn't
reliably supported on native Windows/PowerShell. **Use WSL2** (Windows
Subsystem for Linux) - it gives you a real Ubuntu environment inside
Windows, and every command below then becomes identical to the Mac/Linux
steps.

**Step 1 - Install WSL2 + Ubuntu**

1. Open PowerShell **as Administrator** and run:
   ```powershell
   wsl --install
   ```
2. Restart your computer when prompted.
3. After restart, Ubuntu opens automatically and asks you to create a
   username and password (this is separate from your Windows login - pick
   anything, you'll use it for `sudo`).
4. From now on, open **Ubuntu** from the Start menu (not PowerShell/CMD)
   for every command in this guide.

**Step 2 - Update Ubuntu and install build tools**
```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential pkg-config libssl-dev curl git
```

**Step 3 onward:** follow the **"Mac / Linux (WSL)"** section below
exactly as written - every command is the same inside WSL's Ubuntu shell.

**Editing files:** install [VS Code](https://code.visualstudio.com/) on
the Windows side, then the **WSL extension** (Microsoft) - open your
project with `code .` from inside the Ubuntu terminal and VS Code will
connect to WSL automatically, so you still get a normal Windows GUI editor
against the Linux toolchain.

### Mac / Linux (WSL)

**Step 1 - Node.js**

Use [nvm](https://github.com/nvm-sh/nvm) so you can manage Node versions
cleanly (avoids the classic "installed with sudo, npm install now needs
sudo too" mess):
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc   # or ~/.zshrc if you use zsh (default on modern Mac)
nvm install 20     # Node 20 LTS
nvm use 20
node --version     # should print v20.x.x
npm --version
```
Then install Yarn (the projects in this repo use it):
```bash
npm install -g yarn
```

**Step 2 - Xcode Command Line Tools (Mac only)**

Rust and several native npm packages need a C compiler:
```bash
xcode-select --install
```
(Skip this on WSL/Linux - `build-essential` from the Windows section above
already covers it.)

**Step 3 - Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Press `1` (default install) when prompted, then:
```bash
source "$HOME/.cargo/env"
rustc --version    # e.g. rustc 1.85.0
cargo --version
```

**Step 4 - Solana CLI**
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```
The installer prints a line telling you to add it to your `PATH` - do
that, then restart your terminal (or `source` your shell config) and
confirm:
```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version   # e.g. solana-cli 2.1.16
```
Add that `export PATH=...` line to your `~/.bashrc` / `~/.zshrc` so it
persists across terminal sessions.

**Step 5 - Anchor (via AVM, the Anchor Version Manager)**
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
anchor --version   # anchor-cli 0.30.1
```
This step compiles Anchor from source, so it's the slowest install here -
expect several minutes.

### Verify everything at once

Run this after finishing either path above - all six should print a
version, not "command not found":
```bash
node --version && npm --version && yarn --version && \
rustc --version && cargo --version && \
solana --version && anchor --version
```

### Common install problems

| Symptom | Fix |
|---|---|
| `command not found` right after installing something | Open a **new** terminal tab/window, or run `source ~/.bashrc` (`~/.zshrc` on Mac zsh) - installers edit your shell config file, which only re-loads in new sessions |
| `avm install 0.30.1` fails with a linker/compiler error | Re-run Step 2 (Xcode Command Line Tools) / Step 2 apt packages - Anchor compiles from source and needs a working C toolchain |
| `wsl --install` says WSL is already installed but Ubuntu won't open | Run `wsl --install -d Ubuntu` from an admin PowerShell to install just the Ubuntu distro |
| Node/npm permission errors (`EACCES`) | You installed Node with `sudo`/the OS package manager instead of `nvm` - uninstall it and redo Step 1 with `nvm` |
| `anchor build` fails with `edition2024` or an `rustc ... is not supported` error | This is a separate, expected issue - see "The toolchain gotcha every student will hit" further down this README, not an installation problem |

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
