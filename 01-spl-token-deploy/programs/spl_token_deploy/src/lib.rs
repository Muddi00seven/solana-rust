use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
    Metadata,
};
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
    pub fn create_token(
        ctx: Context<CreateToken>,
        decimals: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(name.chars().count() <= 32, TokenDeployError::NameTooLong);
        require!(symbol.chars().count() <= 10, TokenDeployError::SymbolTooLong);
        require!(uri.chars().count() <= 200, TokenDeployError::UriTooLong);

        // Grab everything we need as plain values *before* taking a mutable
        // borrow of mint_authority - otherwise that borrow stays alive for
        // the rest of the function (its last use is the msg! at the bottom)
        // and blocks every other `ctx.accounts.mint_authority...` access in
        // between (classic borrow-checker vs. Anchor's Context shape).
        let admin_key = ctx.accounts.admin.key();
        let mint_key = ctx.accounts.mint.key();
        let bump = ctx.bumps.mint_authority;

        let mint_authority = &mut ctx.accounts.mint_authority;
        mint_authority.admin = admin_key;
        mint_authority.mint = mint_key;
        mint_authority.bump = bump;
        let mint_authority_key = mint_authority.key();
        // `mint_authority` (the &mut binding) isn't touched again after this
        // point, so its borrow ends here - everything below uses fresh
        // `ctx.accounts...` field accesses instead.

        // The base SPL Mint account has no name/symbol field at all - that's
        // Metaplex Token Metadata's job. We CPI into it here so the token
        // gets a real name/symbol the moment it's created. `mint_authority`
        // is a PDA (no private key), so - same trick as mint_to_wallet - we
        // sign this CPI with its seeds instead of a wallet signature.
        let seeds: &[&[u8]] = &[b"mint_authority", mint_key.as_ref(), &[bump]];
        let signer_seeds: &[&[&[u8]]] = &[seeds];

        let data = DataV2 {
            name: name.clone(),
            symbol: symbol.clone(),
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    mint_authority: ctx.accounts.mint_authority.to_account_info(),
                    payer: ctx.accounts.admin.to_account_info(),
                    // admin (a real wallet, already a signer on this tx) is the
                    // update authority, NOT the PDA - so you can rename/update
                    // this token later straight from a wallet or the Metaplex
                    // CLI without ever touching this program again.
                    update_authority: ctx.accounts.admin.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                signer_seeds,
            ),
            data,
            true,  // is_mutable - you can update name/symbol/uri later
            true,  // update_authority_is_signer
            None,  // collection_details - None = not part of a Metaplex Collection (that's an NFT-collection concept)
        )?;

        msg!(
            "Created mint {} ('{}' / {}) with {} decimals, authority PDA {}",
            mint_key,
            name,
            symbol,
            decimals,
            mint_authority_key
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
#[instruction(decimals: u8, name: String, symbol: String, uri: String)]
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

    /// CHECK: this is the Metaplex Metadata PDA for `mint`. We don't
    /// deserialize it ourselves - the `create_metadata_accounts_v3` CPI
    /// (owned by `token_metadata_program`) validates and initializes it. We
    /// only need to prove *we* derived the same address Metaplex expects.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>,
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
    #[msg("Name must be at most 32 characters (Metaplex's on-chain limit)")]
    NameTooLong,
    #[msg("Symbol must be at most 10 characters (Metaplex's on-chain limit)")]
    SymbolTooLong,
    #[msg("URI must be at most 200 characters (Metaplex's on-chain limit)")]
    UriTooLong,
}
