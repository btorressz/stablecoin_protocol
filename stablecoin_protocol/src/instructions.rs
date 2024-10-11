// instructions.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, Transfer, TokenAccount, Mint, Token};

use crate::state::*;
use crate::errors::*;
use crate::errors::ErrorCode;

// -------------------------------------
// Initialization Instructions
// -------------------------------------

/// Initialize the protocol with the given collateral ratio.
pub fn initialize(ctx: Context<Initialize>, collateral_ratio: u64) -> Result<()> {
    require!(collateral_ratio > 100, ErrorCode::InvalidCollateralRatio); // Ensure reasonable collateral ratio

    let governance = &mut ctx.accounts.governance;
    governance.collateral_ratio = collateral_ratio;

    // Emit an event for the protocol initialization
    emit!(ProtocolInitialized {
        collateral_ratio,
    });

    Ok(())
}

// -------------------------------------
// Minting and Burning Instructions
// -------------------------------------

/// Mint stablecoin with a dynamic fee based on the current price.
pub fn mint_stablecoin(ctx: Context<MintStablecoin>, amount: u64, current_price: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(current_price > 0, ErrorCode::InvalidPrice);

    let user_account = &mut ctx.accounts.user_account;
    let mint = &ctx.accounts.stablecoin_mint;

    // Calculate minting fee based on the price of the stablecoin
    let mut fee = amount / 100; // Default 1% fee
    if current_price > 100 {
        fee /= 2; // Reduce fee if the stablecoin price is above $1.00
    }

    // Ensure the user has enough collateral to mint the stablecoin
    let total_amount = amount + fee;
    let required_collateral = total_amount
        .checked_mul(user_account.collateral_ratio)
        .ok_or(ErrorCode::Overflow)?;
    require!(
        user_account.collateral_balance >= required_collateral,
        ErrorCode::InsufficientCollateral
    );

    // Mint the stablecoin excluding the fee
    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        to: ctx.accounts.user_stablecoin_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::mint_to(cpi_ctx, amount)?;

    // Update the user’s stablecoin balance
    user_account.stablecoin_balance = user_account
        .stablecoin_balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    // Mint the fee to a treasury or governance account
    let cpi_accounts_fee = MintTo {
        mint: mint.to_account_info(),
        to: ctx.accounts.treasury_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_ctx_fee = CpiContext::new(cpi_program, cpi_accounts_fee);
    token::mint_to(cpi_ctx_fee, fee)?;

    // Emit an event for the minting action
    emit!(MintStablecoinEvent {
        user: ctx.accounts.user_account.key(),
        amount,
        fee,
    });

    Ok(())
}

// -------------------------------------
// Liquidation Instructions
// -------------------------------------

/// Partially liquidate a user's under-collateralized position.
pub fn partial_liquidate(ctx: Context<Liquidate>, liquidation_amount: u64) -> Result<()> {
    require!(liquidation_amount > 0, ErrorCode::InvalidAmount);

    let user_account = &mut ctx.accounts.user_account;

    // Check if the user is under-collateralized
    let current_ratio = (user_account.collateral_balance * 100) / user_account.stablecoin_balance;
    require!(
        current_ratio < user_account.collateral_ratio,
        ErrorCode::NotEligibleForLiquidation
    );

    // Calculate the liquidation penalty (e.g., 10%)
    let penalty = liquidation_amount / 10;
    let remaining_collateral = liquidation_amount.checked_sub(penalty).ok_or(ErrorCode::Overflow)?;

    // Deduct the stablecoin and collateral from the user's account
    user_account.stablecoin_balance = user_account.stablecoin_balance
        .checked_sub(liquidation_amount)
        .ok_or(ErrorCode::Overflow)?;

    user_account.collateral_balance = user_account.collateral_balance
        .checked_sub(remaining_collateral)
        .ok_or(ErrorCode::Overflow)?;

    // Transfer the penalty to the liquidator's account
    ctx.accounts.liquidator_collateral_account.amount += penalty;

    // Emit an event for the liquidation
    emit!(LiquidationEvent {
        user: ctx.accounts.user_account.key(),
        amount: liquidation_amount,
        penalty,
    });

    Ok(())
}

// -------------------------------------
// Staking Instructions
// -------------------------------------

/// Stake tokens to earn rewards with lock-up periods.
pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, lockup_period: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(lockup_period > 0, ErrorCode::InvalidLockupPeriod);

    let staker_account = &mut ctx.accounts.staker_account;
    staker_account.staked_balance = staker_account.staked_balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    staker_account.lockup_period = lockup_period;
    staker_account.early_withdrawal_penalty = if lockup_period > 30 * 24 * 60 * 60 { 5 } else { 2 };

    // Transfer the tokens to the staking pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.staking_pool.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Emit an event for the staking action
    emit!(StakeEvent {
        user: ctx.accounts.user_token_account.key(),
        amount,
    });

    Ok(())
}

/// Withdraw staked tokens with optional early withdrawal penalty.
pub fn withdraw_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let staker_account = &mut ctx.accounts.staker_account;
    let current_time = ctx.accounts.clock.unix_timestamp as u64;
    let penalty = if current_time < staker_account.lockup_period {
        amount * staker_account.early_withdrawal_penalty / 100
    } else {
        0
    };

    let final_amount = amount.checked_sub(penalty).ok_or(ErrorCode::Overflow)?;

    // Transfer the staked tokens back to the user
    let cpi_accounts = Transfer {
        from: ctx.accounts.staking_pool.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, final_amount)?;

    // Update the staked balance
    staker_account.staked_balance = staker_account.staked_balance.checked_sub(amount).ok_or(ErrorCode::Overflow)?;

    // Emit an event for the withdrawal
    emit!(WithdrawStakeEvent {
        user: ctx.accounts.user_token_account.key(),
        amount,
        penalty,
    });

    Ok(())
}

// -------------------------------------
// Governance Instructions
// -------------------------------------

/// Create a new governance proposal.
pub fn create_proposal(ctx: Context<CreateProposal>, description: String, new_collateral_ratio: Option<u64>, new_reward_rate: Option<u64>) -> Result<()> {
    require!(description.len() <= 200, ErrorCode::DescriptionTooLong);

    // Make sure at least one change is proposed
    require!(
        new_collateral_ratio.is_some() || new_reward_rate.is_some(),
        ErrorCode::ProposalNoChangesSpecified
    );

    let proposal = &mut ctx.accounts.proposal;
    proposal.description = description;
    proposal.new_collateral_ratio = new_collateral_ratio;
    proposal.new_reward_rate = new_reward_rate;
    proposal.approval_votes = 0;
    proposal.reject_votes = 0;
    proposal.status = ProposalStatus::Pending;
    proposal.proposer = *ctx.accounts.proposer.key;

    // Emit an event for the proposal creation
    emit!(ProposalCreatedEvent {
        proposer: *ctx.accounts.proposer.key,
        proposal_id: *ctx.accounts.proposal.to_account_info().key,
    });

    Ok(())
}

/// Vote on an existing proposal.
pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, approve: bool) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    require!(proposal.status == ProposalStatus::Pending, ErrorCode::ProposalAlreadyConcluded);

    if approve {
        proposal.approval_votes += 1;
    } else {
        proposal.reject_votes += 1;
    }

    // Update proposal status if the vote threshold is reached
    if proposal.approval_votes > proposal.reject_votes {
        proposal.status = ProposalStatus::Approved;
    } else {
        proposal.status = ProposalStatus::Rejected;
    }

    // Apply the changes if the proposal is approved
    if proposal.status == ProposalStatus::Approved {
        if let Some(new_collateral_ratio) = proposal.new_collateral_ratio {
            ctx.accounts.governance.collateral_ratio = new_collateral_ratio;
        }
        if let Some(new_reward_rate) = proposal.new_reward_rate {
            ctx.accounts.governance.reward_adjustment_rate = new_reward_rate;
        }
    }

    // Emit an event for the voting action
    emit!(ProposalVotedEvent {
        voter: *ctx.accounts.voter.key,
        proposal_id: *ctx.accounts.proposal.to_account_info().key,
        approved: approve,
    });

    Ok(())
}

// -------------------------------------
// Multi-collateral Instructions
// -------------------------------------

/// Add a new collateral type to the protocol.
pub fn add_collateral_type(ctx: Context<AddCollateralType>, collateral_ratio: u64) -> Result<()> {
    require!(collateral_ratio > 100, ErrorCode::InvalidCollateralRatio);

    let collateral_type = &mut ctx.accounts.collateral_type;
    collateral_type.collateral_mint = *ctx.accounts.collateral_type.to_account_info().key;
    collateral_type.collateral_ratio = collateral_ratio;
    collateral_type.price_feed = *ctx.accounts.collateral_type.to_account_info().key;

    // Emit an event for adding a new collateral type
    emit!(CollateralTypeAddedEvent {
        collateral_mint: collateral_type.collateral_mint,
        collateral_ratio,
    });

    Ok(())
}

/// Mint stablecoin using a specified collateral type.
pub fn mint_stablecoin_with_collateral(ctx: Context<MintStablecoinWithCollateral>, amount: u64, collateral_type: Pubkey) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let user_account = &mut ctx.accounts.user_account;
    let collateral_type_account = &ctx.accounts.collateral_type;

    // Ensure the specified collateral type matches
    require!(collateral_type_account.collateral_mint == collateral_type, ErrorCode::InvalidCollateralType);

    // Check if the user has enough collateral based on the collateral type's ratio
    let required_collateral = amount.checked_mul(collateral_type_account.collateral_ratio).ok_or(ErrorCode::Overflow)?;
    require!(user_account.collateral_balance >= required_collateral, ErrorCode::InsufficientCollateral);

    // Mint stablecoins
    let cpi_accounts = MintTo {
        mint: ctx.accounts.stablecoin_mint.to_account_info(),
        to: ctx.accounts.user_stablecoin_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::mint_to(cpi_ctx, amount)?;

    // Update the user's stablecoin balance
    user_account.stablecoin_balance = user_account.stablecoin_balance.checked_add(amount).ok_or(ErrorCode::Overflow)?;

    // Emit an event for minting stablecoin with collateral
    emit!(MintStablecoinWithCollateralEvent {
        user: ctx.accounts.user_account.key(),
        amount,
        collateral_type,
    });

    Ok(())
}

// -------------------------------------
// Claim Rewards (Implementation)
// -------------------------------------

/// Claim staking rewards.
pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let staker_account = &mut ctx.accounts.staker_account;
    let current_time = Clock::get()?.unix_timestamp as u64;

    // Calculate rewards
    let time_since_last_claim = current_time.checked_sub(staker_account.last_reward_claim).ok_or(ErrorCode::Overflow)?;
    let reward_amount = (staker_account.staked_balance * time_since_last_claim) / 1_000_000; // Example calculation

    // Update last reward claim time
    staker_account.last_reward_claim = current_time;

    // Mint the rewards
    let cpi_accounts = MintTo {
        mint: ctx.accounts.reward_token_mint.to_account_info(),
        to: ctx.accounts.user_reward_account.to_account_info(),
        authority: ctx.accounts.reward_mint_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::mint_to(cpi_ctx, reward_amount)?;

    Ok(())
}

// -------------------------------------
// Event Definitions
// -------------------------------------

#[event]
pub struct ProtocolInitialized {
    pub collateral_ratio: u64,
}

#[event]
pub struct MintStablecoinEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub fee: u64,
}

#[event]
pub struct LiquidationEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub penalty: u64,
}

#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawStakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub penalty: u64,
}

#[event]
pub struct ProposalCreatedEvent {
    pub proposer: Pubkey,
    pub proposal_id: Pubkey,
}

#[event]
pub struct ProposalVotedEvent {
    pub voter: Pubkey,
    pub proposal_id: Pubkey,
    pub approved: bool,
}

#[event]
pub struct CollateralTypeAddedEvent {
    pub collateral_mint: Pubkey,
    pub collateral_ratio: u64,
}

#[event]
pub struct MintStablecoinWithCollateralEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub collateral_type: Pubkey,
}
