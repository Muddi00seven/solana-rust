use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

// After your first `anchor build`, run `anchor keys list` and paste the real
// program id here AND in Anchor.toml (or just run `anchor keys sync`).
declare_id!("237gzJeQzJkTx9Zsb5zFAuJVjdCr5yetGtUDoikde53s");

#[program]
pub mod spl_token_deploy {
    use super::*;

    /// Creates a brand new SPL token mint. The mint authority is a PDA
    /// ("mint_authority") controlled by this program, not a raw wallet -
    /// that's the whole point of doing this via Anchor instead of just
    /// running `spl-token create-token` from the CLI: minting logic can now
    /// be gated by whatever program rules you want.
    pub fn create_token(ctx: Context<CreateToken>, decimals: u8) -> Result<()> {
        let mint_authority = &mut ctx.accounts.mint_authority;
        mint_authority.admin = ctx.accounts.admin.key();
        mint_authority.mint = ctx.accounts.mint.key();
        mint_authority.bump = ctx.bumps.mint_authority;

        msg!(
            "Created mint {} with {} decimals, authority PDA {}",
            ctx.accounts.mint.key(),
            decimals,
            mint_authority.key()
        );
        Ok(())
    }

    /// Mints `amount` base units of the token to `recipient`'s associated
    /// token account. Only the `admin` stored on the MintAuthority PDA
    /// (set once in create_token) is allowed to call this.
    pub fn mint_to_wallet(ctx: Context<MintToWallet>, amount: u64) -> Result<()> {
        require!(amount > 0, TokenDeployError::ZeroAmount);

        let mint_key = ctx.accounts.mint.key();
        let seeds: &[&[u8]] = &[
            b"mint_authority",
            mint_key.as_ref(),
            &[ctx.accounts.mint_authority.bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[seeds];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::mint_to(cpi_ctx, amount)?;

        msg!(
            "Minted {} base units to {} (owner {})",
            amount,
            ctx.accounts.recipient_token_account.key(),
            ctx.accounts.recipient.key()
        );
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The mint account itself. `admin` generates a fresh Keypair client-side
    /// for this (see scripts/create-token.ts) and passes its pubkey in - it
    /// is NOT a PDA, so Anchor just creates it as a normal account.
    #[account(
        init,
        payer = admin,
        mint::decimals = decimals,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    /// PDA that becomes the mint's authority. Storing `admin` here is what
    /// lets mint_to_wallet check "is the caller allowed to mint more?".
    #[account(
        init,
        payer = admin,
        space = 8 + MintAuthority::INIT_SPACE,
        seeds = [b"mint_authority", mint.key().as_ref()],
        bump,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintToWallet<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    // `admin` is declared above this field, so referencing it here in
    // `has_one` is safe - Anchor binds accounts top-to-bottom and this
    // constraint just checks mint_authority.admin == admin.key().
    #[account(
        mut,
        seeds = [b"mint_authority", mint.key().as_ref()],
        bump = mint_authority.bump,
        has_one = mint,
        has_one = admin @ TokenDeployError::NotAdmin,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// The wallet that will own the tokens. Doesn't need to sign - anyone
    /// can be minted to, only `admin` needs to authorize the mint itself.
    /// CHECK: only used as the owner of `recipient_token_account`; no data
    /// is read from it.
    pub recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct MintAuthority {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum TokenDeployError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Only the admin who created this token can mint more of it")]
    NotAdmin,
}
