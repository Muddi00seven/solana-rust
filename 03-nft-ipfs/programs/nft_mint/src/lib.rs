use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3,
    mpl_token_metadata::types::DataV2, CreateMasterEditionV3, CreateMetadataAccountsV3, Metadata,
};
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

declare_id!("AXfnZ45FUoVyzTZTNdkm6XG9QzwYjrfk7vomKEobuVXF");

#[program]
pub mod nft_mint {
    use super::*;

    /// Mints a genuine 1-of-1 NFT in a single instruction:
    ///   1. creates a fresh Mint with 0 decimals
    ///   2. mints exactly 1 unit of it into the payer's associated token account
    ///   3. CPIs into the Metaplex Token Metadata program to attach
    ///      name/symbol/uri (the `uri` should point at a JSON file on IPFS -
    ///      see scripts/upload-to-ipfs.ts)
    ///   4. CPIs again to create a Master Edition, which is what actually
    ///      marks this mint as non-fungible/uneditionable (max_supply = 0)
    ///      rather than just "a token that happens to have 0 decimals"
    pub fn mint_nft(ctx: Context<MintNft>, name: String, symbol: String, uri: String) -> Result<()> {
        require!(name.chars().count() <= 32, NftError::NameTooLong);
        require!(symbol.chars().count() <= 10, NftError::SymbolTooLong);
        require!(uri.chars().count() <= 200, NftError::UriTooLong);

        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            1,
        )?;

        let data = DataV2 {
            name: name.clone(),
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            data,
            true,  // is_mutable - keep true while teaching so mistakes are fixable
            true,  // update_authority_is_signer
            None,  // collection_details - not part of a Metaplex Collection
        )?;

        create_master_edition_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            Some(0), // max_supply = 0 -> no additional "prints" of this edition
        )?;

        msg!("Minted NFT '{}' -> mint {}", name, ctx.accounts.mint.key());
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// A fresh Keypair generated client-side per NFT (see
    /// scripts/mint-nft.ts) - each NFT is its own unique mint with a
    /// hard-capped supply of 1.
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer,
        mint::freeze_authority = payer,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    /// CHECK: this is the Metaplex Metadata PDA. We don't deserialize it
    /// ourselves - the `create_metadata_accounts_v3` CPI (owned by
    /// `token_metadata_program`) validates and initializes it. We only need
    /// to prove *we* derived the same address Metaplex expects.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: same idea as `metadata`, but for the Master Edition PDA.
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition",
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub master_edition: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum NftError {
    #[msg("Name must be at most 32 characters (Metaplex's on-chain limit)")]
    NameTooLong,
    #[msg("Symbol must be at most 10 characters (Metaplex's on-chain limit)")]
    SymbolTooLong,
    #[msg("URI must be at most 200 characters (Metaplex's on-chain limit)")]
    UriTooLong,
}
