use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

// This program ports the logic of a reference Solidity crowdfunding
// contract (Campaign struct with title/description/goal/deadline, a
// Contribution struct per donor, create/contribute/withdraw) onto Solana,
// swapping ERC20-approve-then-transferFrom for a direct SPL token
// `transfer` CPI, and swapping a `Campaign[]` array for one PDA account per
// campaign. It also adds a `claim_refund` instruction the original
// contract didn't have - donors can get their money back if a campaign
// expires without hitting its goal.
declare_id!("6hafcYNa34it4syjPPy8yuQW5LfCmcxhm3D78h2qBA5r");

pub const SECONDS_PER_DAY: i64 = 86_400;

#[program]
pub mod crowdfunding {
    use super::*;

    /// Creates a new funding pool. `campaign_id` is any number the creator
    /// picks (e.g. a client-side counter or a timestamp) - it exists purely
    /// so one wallet can run multiple campaigns, since each PDA needs a
    /// unique seed.
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        campaign_id: u64,
        title: String,
        description: String,
        image_url: String,
        goal: u64,
        duration_days: i64,
    ) -> Result<()> {
        require!(
            title.chars().count() >= 5 && title.chars().count() <= 64,
            CrowdfundingError::InvalidTitle
        );
        require!(
            description.chars().count() <= 200,
            CrowdfundingError::DescriptionTooLong
        );
        require!(
            image_url.chars().count() <= 200,
            CrowdfundingError::ImageUrlTooLong
        );
        require!(goal > 0, CrowdfundingError::InvalidGoal);
        require!(
            duration_days >= 1 && duration_days <= 365,
            CrowdfundingError::InvalidDuration
        );

        let now = Clock::get()?.unix_timestamp;
        let creator_key = ctx.accounts.creator.key();
        let token_mint_key = ctx.accounts.token_mint.key();

        let campaign = &mut ctx.accounts.campaign;
        campaign.creator = creator_key;
        campaign.campaign_id = campaign_id;
        campaign.title = title.clone();
        campaign.description = description;
        campaign.image_url = image_url;
        campaign.token_mint = token_mint_key;
        campaign.goal = goal;
        campaign.raised = 0;
        campaign.deadline = now
            .checked_add(duration_days.checked_mul(SECONDS_PER_DAY).ok_or(CrowdfundingError::MathOverflow)?)
            .ok_or(CrowdfundingError::MathOverflow)?;
        campaign.withdrawn = false;
        campaign.contributors_count = 0;
        campaign.bump = ctx.bumps.campaign;
        campaign.vault_bump = ctx.bumps.vault;

        emit!(CampaignCreated {
            campaign: campaign.key(),
            creator: creator_key,
            title,
            goal,
            deadline: campaign.deadline,
        });
        Ok(())
    }

    /// Transfers `amount` of the campaign's token from the contributor's
    /// own token account into the campaign vault. Unlike the ERC20
    /// reference contract, there's no separate "approve" step - SPL token
    /// transfers are authorized directly by the owner's signature.
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        require!(amount > 0, CrowdfundingError::ZeroAmount);
        let now = Clock::get()?.unix_timestamp;
        require!(now < ctx.accounts.campaign.deadline, CrowdfundingError::CampaignExpired);

        let campaign_key = ctx.accounts.campaign.key();
        let contributor_key = ctx.accounts.contributor.key();

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.contributor_token_account.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.contributor.to_account_info(),
                },
            ),
            amount,
        )?;

        let is_new_contributor = ctx.accounts.contribution.amount == 0;

        let contribution = &mut ctx.accounts.contribution;
        contribution.campaign = campaign_key;
        contribution.contributor = contributor_key;
        contribution.amount = contribution
            .amount
            .checked_add(amount)
            .ok_or(CrowdfundingError::MathOverflow)?;
        contribution.last_contributed_at = now;
        contribution.bump = ctx.bumps.contribution;

        let campaign = &mut ctx.accounts.campaign;
        campaign.raised = campaign
            .raised
            .checked_add(amount)
            .ok_or(CrowdfundingError::MathOverflow)?;
        if is_new_contributor {
            campaign.contributors_count = campaign
                .contributors_count
                .checked_add(1)
                .ok_or(CrowdfundingError::MathOverflow)?;
        }

        emit!(ContributionMade {
            campaign: campaign_key,
            contributor: contributor_key,
            amount,
            total_raised: campaign.raised,
        });
        Ok(())
    }

    /// Creator pulls the raised funds out, once and only once, and only if
    /// the goal was met. Signed by the `campaign` PDA itself via its own
    /// stored seeds - the vault's tokens are controlled entirely by program
    /// logic, not by any human-held key.
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        require!(
            ctx.accounts.campaign.raised >= ctx.accounts.campaign.goal,
            CrowdfundingError::GoalNotReached
        );
        require!(!ctx.accounts.campaign.withdrawn, CrowdfundingError::AlreadyWithdrawn);

        let amount = ctx.accounts.campaign.raised;
        let creator_key = ctx.accounts.campaign.creator;
        let campaign_id = ctx.accounts.campaign.campaign_id;
        let bump = ctx.accounts.campaign.bump;
        let campaign_id_bytes = campaign_id.to_le_bytes();
        let signer_seeds: &[&[u8]] = &[
            b"campaign",
            creator_key.as_ref(),
            &campaign_id_bytes,
            &[bump],
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.creator_token_account.to_account_info(),
                    authority: ctx.accounts.campaign.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;

        let campaign = &mut ctx.accounts.campaign;
        campaign.withdrawn = true;

        emit!(FundsWithdrawn {
            campaign: campaign.key(),
            creator: creator_key,
            amount,
        });
        Ok(())
    }

    /// Bonus instruction not present in the reference Solidity contract:
    /// if a campaign's deadline passes without hitting its goal, each
    /// contributor can pull their own money back out instead of it being
    /// stuck in the vault forever.
    pub fn claim_refund(ctx: Context<ClaimRefund>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(now >= ctx.accounts.campaign.deadline, CrowdfundingError::CampaignStillActive);
        require!(
            ctx.accounts.campaign.raised < ctx.accounts.campaign.goal,
            CrowdfundingError::GoalWasReached
        );
        require!(!ctx.accounts.contribution.refunded, CrowdfundingError::AlreadyRefunded);
        require!(ctx.accounts.contribution.amount > 0, CrowdfundingError::NothingToRefund);

        let amount = ctx.accounts.contribution.amount;
        let creator_key = ctx.accounts.campaign.creator;
        let campaign_id = ctx.accounts.campaign.campaign_id;
        let bump = ctx.accounts.campaign.bump;
        let campaign_id_bytes = campaign_id.to_le_bytes();
        let signer_seeds: &[&[u8]] = &[
            b"campaign",
            creator_key.as_ref(),
            &campaign_id_bytes,
            &[bump],
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.contributor_token_account.to_account_info(),
                    authority: ctx.accounts.campaign.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;

        let contribution = &mut ctx.accounts.contribution;
        contribution.refunded = true;

        emit!(RefundClaimed {
            campaign: ctx.accounts.campaign.key(),
            contributor: contribution.contributor,
            amount,
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE,
        seeds = [b"campaign", creator.key().as_ref(), &campaign_id.to_le_bytes()],
        bump,
    )]
    pub campaign: Account<'info, Campaign>,

    /// The vault is itself a PDA-owned token account. Its *address* is
    /// derived from seeds, but the *authority* that can move its tokens is
    /// the `campaign` account (also a PDA) - see the signer_seeds in
    /// `withdraw` / `claim_refund`.
    #[account(
        init,
        payer = creator,
        seeds = [b"vault", campaign.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = campaign,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign", campaign.creator.as_ref(), &campaign.campaign_id.to_le_bytes()],
        bump = campaign.bump,
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = contributor,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", campaign.key().as_ref(), contributor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        mut,
        associated_token::mint = campaign.token_mint,
        associated_token::authority = contributor,
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign", campaign.creator.as_ref(), &campaign.campaign_id.to_le_bytes()],
        bump = campaign.bump,
        has_one = creator @ CrowdfundingError::NotCreator,
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    // `associated_token::mint = ...` on an `init_if_needed` account needs an
    // actual `Account<'info, Mint>` field to CPI against - a bare Pubkey
    // expression like `campaign.token_mint` (fine for read-only address
    // checks elsewhere in this file) doesn't work here. We still confirm it
    // really is the campaign's mint via the `address` constraint below.
    #[account(address = campaign.token_mint)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = token_mint,
        associated_token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimRefund<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(
        seeds = [b"campaign", campaign.creator.as_ref(), &campaign.campaign_id.to_le_bytes()],
        bump = campaign.bump,
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump = campaign.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"contribution", campaign.key().as_ref(), contributor.key().as_ref()],
        bump = contribution.bump,
        has_one = contributor,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        mut,
        associated_token::mint = campaign.token_mint,
        associated_token::authority = contributor,
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct Campaign {
    pub creator: Pubkey,
    pub campaign_id: u64,
    #[max_len(64)]
    pub title: String,
    #[max_len(200)]
    pub description: String,
    #[max_len(200)]
    pub image_url: String,
    pub token_mint: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub withdrawn: bool,
    pub contributors_count: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Contribution {
    pub campaign: Pubkey,
    pub contributor: Pubkey,
    pub amount: u64,
    pub last_contributed_at: i64,
    pub refunded: bool,
    pub bump: u8,
}

#[event]
pub struct CampaignCreated {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub title: String,
    pub goal: u64,
    pub deadline: i64,
}

#[event]
pub struct ContributionMade {
    pub campaign: Pubkey,
    pub contributor: Pubkey,
    pub amount: u64,
    pub total_raised: u64,
}

#[event]
pub struct FundsWithdrawn {
    pub campaign: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RefundClaimed {
    pub campaign: Pubkey,
    pub contributor: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum CrowdfundingError {
    #[msg("Title must be between 5 and 64 characters")]
    InvalidTitle,
    #[msg("Description must be at most 200 characters")]
    DescriptionTooLong,
    #[msg("Image URL must be at most 200 characters")]
    ImageUrlTooLong,
    #[msg("Goal must be greater than zero")]
    InvalidGoal,
    #[msg("Duration must be between 1 and 365 days")]
    InvalidDuration,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("This campaign's deadline has passed")]
    CampaignExpired,
    #[msg("Only the campaign creator can withdraw")]
    NotCreator,
    #[msg("The funding goal has not been reached yet")]
    GoalNotReached,
    #[msg("Funds have already been withdrawn")]
    AlreadyWithdrawn,
    #[msg("Campaign is still active - wait for the deadline to claim a refund")]
    CampaignStillActive,
    #[msg("The funding goal was reached - ask the creator to withdraw instead")]
    GoalWasReached,
    #[msg("This contribution has already been refunded")]
    AlreadyRefunded,
    #[msg("Nothing to refund for this contributor")]
    NothingToRefund,
    #[msg("Math overflow")]
    MathOverflow,
}
