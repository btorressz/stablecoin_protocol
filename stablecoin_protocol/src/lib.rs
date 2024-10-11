use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::clock::Clock;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use state::{Initialize, MintStablecoin, MintStablecoinWithCollateral, Liquidate, StakeTokens, WithdrawStake, ClaimRewards, ProposalStatus, CreateProposal, VoteOnProposal, AddCollateralType};
use errors::ErrorCode;

declare_id!("2oNrfjvaXeRCcU82pMQLN4guMR4jfZsCJLgpKNuCfYDP");

#[program]
pub mod stablecoin_protocol {
    use super::*;

    // -------------------------------------
    // Initialization Functions
    // -------------------------------------

    /// Initialize the protocol with the given collateral ratio.
    pub fn initialize(ctx: Context<Initialize>, collateral_ratio: u64) -> Result<()> {
        require!(collateral_ratio > 100, ErrorCode::InvalidCollateralRatio); // Ensure collateral ratio is reasonable
        instructions::initialize(ctx, collateral_ratio)
    }

    // -------------------------------------
    // Minting and Burning Functions
    // -------------------------------------

    /// Mint stablecoin with dynamic fee based on the current price.
    pub fn mint_stablecoin(ctx: Context<MintStablecoin>, amount: u64, current_price: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount); // Ensure non-zero minting amount
        require!(current_price > 0, ErrorCode::InvalidPrice); // Ensure valid current price

        // Perform access control to restrict minting to only authorized accounts (if needed)
        if let Some(authority) = ctx.accounts.optional_authority {
            require_keys_eq!(authority.key(), ctx.accounts.user_account.key(), ErrorCode::UnauthorizedOperation);
        }

        instructions::mint_stablecoin(ctx, amount, current_price)
    }

    /// Mint stablecoin using a specified collateral type.
    pub fn mint_stablecoin_with_collateral(ctx: Context<MintStablecoinWithCollateral>, amount: u64, collateral_type: Pubkey) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount); // Ensure non-zero minting amount

        // Access control to restrict minting to authorized users if necessary
        if let Some(authority) = ctx.accounts.optional_authority {
            require_keys_eq!(authority.key(), ctx.accounts.user_account.key(), ErrorCode::UnauthorizedOperation);
        }

        instructions::mint_stablecoin_with_collateral(ctx, amount, collateral_type)
    }

    // -------------------------------------
    // Liquidation Functions
    // -------------------------------------

    /// Partially liquidate a user's under-collateralized position.
    pub fn partial_liquidate(ctx: Context<Liquidate>, liquidation_amount: u64) -> Result<()> {
        require!(liquidation_amount > 0, ErrorCode::InvalidAmount); // Ensure non-zero liquidation amount

        let user_account = &ctx.accounts.user_account;
        let current_ratio = (user_account.collateral_balance * 100) / user_account.stablecoin_balance;
        require!(current_ratio < user_account.collateral_ratio, ErrorCode::NotEligibleForLiquidation);

        instructions::partial_liquidate(ctx, liquidation_amount)
    }

    // -------------------------------------
    // Staking Functions
    // -------------------------------------

    /// Stake tokens to earn rewards with lock-up periods.
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, lockup_period: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount); // Ensure non-zero staking amount
        require!(lockup_period > 0, ErrorCode::InvalidLockupPeriod); // Ensure valid lock-up period

        instructions::stake_tokens(ctx, amount, lockup_period)
    }

    /// Withdraw staked tokens with optional early withdrawal penalty.
    pub fn withdraw_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount); // Ensure non-zero withdrawal amount

        let staker_account = &ctx.accounts.staker_account;
        let current_time = Clock::get()?.unix_timestamp as u64;
        require!(current_time >= staker_account.lockup_period, ErrorCode::LockupPeriodNotOver); // Ensure lock-up period is over

        instructions::withdraw_stake(ctx, amount)
    }

    /// Claim staking rewards.
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let staker_account = &ctx.accounts.staker_account;
        let current_time = Clock::get()?.unix_timestamp as u64;

        // Ensure that enough time has passed since the last claim
        require!(current_time > staker_account.last_reward_claim, ErrorCode::RewardsAlreadyClaimed);

        instructions::claim_rewards(ctx)
    }

    // -------------------------------------
    // Governance Functions
    // -------------------------------------

    /// Create a new governance proposal.
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        description: String,
        new_collateral_ratio: Option<u64>,
        new_reward_rate: Option<u64>,
    ) -> Result<()> {
        require!(description.len() <= 200, ErrorCode::DescriptionTooLong); // Limit description length

        // Ensure that the proposal changes are meaningful
        if let Some(collateral_ratio) = new_collateral_ratio {
            require!(collateral_ratio > 100, ErrorCode::InvalidCollateralRatio); // Make sure ratio is above 100%
        }

        instructions::create_proposal(ctx, description, new_collateral_ratio, new_reward_rate)
    }

    /// Vote on an existing proposal.
    pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, approve: bool) -> Result<()> {
        let proposal = &ctx.accounts.proposal;
        require!(proposal.status == ProposalStatus::Pending, ErrorCode::ProposalAlreadyConcluded); // Ensure the proposal is still open

        instructions::vote_on_proposal(ctx, approve)
    }

    // -------------------------------------
    // Multi-collateral Functions
    // -------------------------------------

    /// Add a new collateral type to the protocol.
    pub fn add_collateral_type(ctx: Context<AddCollateralType>, collateral_ratio: u64) -> Result<()> {
        require!(collateral_ratio > 100, ErrorCode::InvalidCollateralRatio); // Ensure reasonable collateral ratio

        instructions::add_collateral_type(ctx, collateral_ratio)
    }
}