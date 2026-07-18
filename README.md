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
CLI**, and **Anchor**. Pick the section for your OS below and follow it
top to bottom - they're written to be complete on their own, nothing is
shared between them.

---

## Installing on Windows

Solana's build tooling (`cargo-build-sbf`) is Linux-first and isn't
reliably supported on native Windows/PowerShell, so this guide installs
**WSL2** (Windows Subsystem for Linux) first - it gives you a real Ubuntu
environment inside Windows - and everything after that runs inside it.

**Step 1 - Install WSL2 + Ubuntu**

1. Open PowerShell **as Administrator** and run:
   ```powershell
   wsl --install
   ```
2. Restart your computer when prompted.
3. After restart, Ubuntu opens automatically and asks you to create a
   username and password (separate from your Windows login - pick
   anything, you'll use it for `sudo`).
4. From now on, open **Ubuntu** from the Start menu (not PowerShell/CMD)
   for every command below.

**Step 2 - Update Ubuntu and install build tools**
```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential pkg-config libssl-dev curl git
```

**Step 3 - Node.js**

Use [nvm](https://github.com/nvm-sh/nvm) to manage Node versions cleanly:
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc
nvm install 20     # Node 20 LTS
nvm use 20
node --version     # should print v20.x.x
npm --version
```
Then install Yarn (the projects in this repo use it):
```bash
npm install -g yarn
```

**Step 4 - Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Press `1` (default install) when prompted, then:
```bash
source "$HOME/.cargo/env"
rustc --version    # e.g. rustc 1.85.0
cargo --version
```

**Step 5 - Solana CLI**
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version   # e.g. solana-cli 2.1.16
```
Add that `export PATH=...` line to the end of `~/.bashrc` so it persists
across terminal sessions:
```bash
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
```

**Step 6 - Anchor (via AVM, the Anchor Version Manager)**
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
anchor --version   # anchor-cli 0.30.1
```
This compiles Anchor from source, so it's the slowest step here - expect
several minutes.

**Step 7 - Verify everything**
```bash
node --version && npm --version && yarn --version && \
rustc --version && cargo --version && \
solana --version && anchor --version
```
All seven should print a version, not "command not found."

**Editing files:** install [VS Code](https://code.visualstudio.com/) on
the Windows side, then the **WSL extension** (Microsoft). Open your
project with `code .` from inside the Ubuntu terminal and VS Code
connects to WSL automatically - a normal Windows GUI editor working
against the Linux toolchain you just installed.

**Common Windows/WSL problems**

| Symptom | Fix |
|---|---|
| `command not found` right after installing something | Open a **new** Ubuntu terminal window, or run `source ~/.bashrc` - installers edit your shell config, which only reloads in new sessions |
| `avm install 0.30.1` fails with a linker/compiler error | Re-run Step 2 (`apt install build-essential ...`) - Anchor compiles from source and needs a working C toolchain |
| `wsl --install` says WSL is already installed but Ubuntu won't open | Run `wsl --install -d Ubuntu` from an admin PowerShell to install just the Ubuntu distro |
| Node/npm permission errors (`EACCES`) | You installed Node with `sudo`/`apt` instead of `nvm` - remove it and redo Step 3 with `nvm` |
| `anchor build` fails with `edition2024` or `rustc ... is not supported` | Expected, unrelated to installation - see "The toolchain gotcha every student will hit" further down this README |

---

## Installing on macOS

**Step 1 - Xcode Command Line Tools**

Rust and several native npm packages need a C compiler:
```bash
xcode-select --install
```

**Step 2 - Node.js**

Use [nvm](https://github.com/nvm-sh/nvm) to manage Node versions cleanly
(avoids the classic "installed with the macOS installer, now `npm
install` needs sudo" mess):
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.zshrc    # zsh is the default shell on modern macOS; use ~/.bash_profile if you're on bash
nvm install 20     # Node 20 LTS
nvm use 20
node --version     # should print v20.x.x
npm --version
```
Then install Yarn (the projects in this repo use it):
```bash
npm install -g yarn
```

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
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version   # e.g. solana-cli 2.1.16
```
Add that `export PATH=...` line to the end of `~/.zshrc` so it persists
across terminal sessions:
```bash
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.zshrc
```

**Step 5 - Anchor (via AVM, the Anchor Version Manager)**
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
anchor --version   # anchor-cli 0.30.1
```
This compiles Anchor from source, so it's the slowest step here - expect
several minutes.

**Step 6 - Verify everything**
```bash
node --version && npm --version && yarn --version && \
rustc --version && cargo --version && \
solana --version && anchor --version
```
All seven should print a version, not "command not found."

**Common macOS problems**

| Symptom | Fix |
|---|---|
| `command not found` right after installing something | Open a **new** Terminal tab/window, or run `source ~/.zshrc` - installers edit your shell config, which only reloads in new sessions |
| `xcode-select --install` says tools are already installed, but Rust still fails to link | Run `sudo xcode-select --reset`, then retry |
| `avm install 0.30.1` fails with a linker/compiler error | Re-run Step 1 (Xcode Command Line Tools) - Anchor compiles from source and needs a working C toolchain |
| Node/npm permission errors (`EACCES`) | You installed Node with the macOS `.pkg` installer or Homebrew instead of `nvm` - remove it and redo Step 2 with `nvm` |
| `anchor build` fails with `edition2024` or `rustc ... is not supported` | Expected, unrelated to installation - see "The toolchain gotcha every student will hit" further down this README |

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
