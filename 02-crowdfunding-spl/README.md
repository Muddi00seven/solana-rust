# Project 2 — SPL-Token Crowdfunding Pools

Lecture 2 goal: take a real EVM crowdfunding contract you already built
(`crowd-funding-dapp/contracts/CrowdFunding.sol`, the "ChainFund" project)
and port its logic to Solana - same idea, very different implementation,
which is the best way to make PDAs and CPIs click for students who already
think in Solidity.

## 0. The reference contract, and what changed porting it to Solana

The Solidity version stores `Campaign[]` in one contract's storage and a
`mapping(uint256 => Contribution[])` alongside it. Solana doesn't have
growable on-chain arrays like that - state lives in independent accounts.
So:

| ChainFund (Solidity)                        | This program (Anchor)                                         |
|-----------------------------------------------|-----------------------------------------------------------------|
| `Campaign[] campaigns` (array in contract storage) | One PDA account per campaign: `["campaign", creator, campaign_id]` |
| `mapping(campaignId => Contribution[])`      | One PDA account per (campaign, contributor): `["contribution", campaign, contributor]` |
| ERC20 `approve()` then `transferFrom()`      | Direct SPL `transfer` CPI, signed by the contributor - no approve step |
| Contract itself holds the USDT balance        | A vault token account (PDA-owned) holds the SPL tokens          |
| `require(msg.sender == campaign.creator)`     | `has_one = creator` constraint on the `Campaign` account         |
| No refund path if a campaign fails             | Added `claim_refund` - see "Teaching notes" below                |

Everything else - the goal/deadline/withdrawn checks, the 5-64 char title
rule, the 1-365 day duration rule - is carried over line for line.

## 1. Prerequisites

Same toolchain as project 1 - see `../01-spl-token-deploy/README.md`
section 1 if you haven't installed Rust/Solana CLI/Anchor/Node yet.

## 2. Setup

```bash
cp .env.example .env
node ../01-spl-token-deploy/scripts/keypair-to-base58.js ~/deployer-raw.json
# paste into .env as DEPLOYER_PRIVATE_KEY
yarn install
yarn wallet:from-env
```

Either set `TOKEN_MINT` in `.env` to an existing token (your project 1
token, devnet USDC, etc.), or generate a throwaway one for class:
```bash
yarn setup-test-mint
```

## 3. Build and deploy

```bash
anchor keys list        # note the program id
# paste into declare_id!() in programs/crowdfunding/src/lib.rs and Anchor.toml
# (or run `anchor keys sync`)
anchor build
anchor deploy --provider.cluster devnet
```

## 4. Test locally

```bash
anchor test
```
Covers: creating a campaign, contributing the full goal amount, the
creator withdrawing once, and a second withdraw attempt being rejected.
(`claim_refund` needs a campaign whose deadline has already passed, which
means warping the local validator's clock - noted as a follow-up exercise
in the test file, good for a stronger cohort.)

## 5. Run it end to end on devnet

```bash
yarn create-campaign -- "Solar Panels for Rural Schools" "Installing solar energy" 500 30
yarn contribute -- 100
yarn contribute -- 400          # same wallet again just to hit the goal in a demo
yarn withdraw
```

## Teaching notes

- **`campaign_id` as a seed**: Solidity used an auto-incrementing array
  index; Solana PDAs need seeds decided *before* the account exists, so the
  creator picks an id (the demo script uses `Date.now()`). Good discussion:
  what happens if two transactions from the same wallet pick the same id in
  the same slot? (Answer: the second `init` fails - account already exists
  - which is itself a useful "why PDAs prevent double-creation for free"
  moment.)
- **The vault's authority is the `campaign` PDA, not a separate PDA.**
  Since `campaign` is already a PDA, there was no need to invent another
  "vault_authority" PDA just to hold vault permissions - the campaign
  account signs for itself via its own stored `bump`. Fewer accounts,
  same security.
- **`has_one` ordering bug**, again: in `Withdraw`, `campaign` needs
  `has_one = creator`, and `creator` must be declared *earlier* in the
  struct for that to compile - the exact same rule from project 1.
  Worth drilling since it's the #1 Anchor compile error students will hit
  on their own.
- **`claim_refund` doesn't exist in the Solidity original.** Point this out
  explicitly - it's a real gap in a lot of beginner crowdfunding contracts
  (funds get stuck forever if a campaign fails). Ask the class "how would
  you add this to the Solidity version?" before showing them this file's
  answer - `transferFrom` doesn't even need approval this time since the
  *vault* (not the contributor) is the source, signed by the `campaign`
  PDA the same way `withdraw` does it.

## Verification status

Built end to end with `cargo check` + full `anchor build` on Anchor CLI
`0.30.1` / Solana CLI `2.1.16`, producing a real `crowdfunding.so`. One
real bug was caught here specifically: `associated_token::mint = ...` on
an `init_if_needed` token account (the creator's payout account in
`Withdraw`) needs an actual `Account<'info, Mint>` field to CPI against -
a bare `Pubkey` expression like `campaign.token_mint` (which works fine
for non-init accounts elsewhere in this file) isn't enough there. Fixed by
adding a `token_mint` account field with an `address = campaign.token_mint`
check.

See `../01-spl-token-deploy/README.md`'s "Verification status" section for
the toolchain (`edition2024` / `rustc` version) gotchas you'll likely also
hit running `anchor build` yourself - `Cargo.lock` here already has the
same fixes pinned in.
