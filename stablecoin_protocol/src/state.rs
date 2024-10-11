// state.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

// -------------------------------------
// User Account Structure
// -------------------------------------
#[account]
pub struct UserAccount {
    pub collateral_balance: u64,        // The amount of collateral deposited
    pub stablecoin_balance: u64,        // The amount of stablecoin minted
    pub collateral_ratio: u64,          // The required collateral ratio (e.g., 150%)
    pub last_liquidation_time: u64,     // Timestamp of the last liquidation action
    pub last_mint_time: u64,            // Timestamp of the last minting action
}

// -------------------------------------
// Governance Structure
// -------------------------------------
#[account]
pub struct Governance {
    pub collateral_ratio: u64,          // Global collateral ratio for the protocol
    pub volatility_threshold: u64,      // Threshold to adjust collateral ratio
    pub reward_adjustment_rate: u64,    // Rate for adjusting rewards based on proposals
    pub minimum_approval_threshold: u32, // Minimum number of approval votes needed
}

// -------------------------------------
// Staker Account Structure
// -------------------------------------
#[account]
pub struct StakerAccount {
    pub staked_balance: u64,            // The amount of tokens staked by the user
    pub last_reward_claim: u64,         // Timestamp of the last reward claim
    pub reward_debt: u64,               // Accumulated rewards not yet claimed
    pub lockup_period: u64,             // Lock-up period in seconds
    pub early_withdrawal_penalty: u64,  // Penalty for withdrawing before lock-up period
    pub reward_multiplier: u64,         // Multiplier for calculating rewards (based on lock-up or staking duration)
    pub auto_compound: bool,            // Indicates if rewards should be auto-compounded
}

// -------------------------------------
// Reward Pool Structure
// -------------------------------------
#[account]
pub struct RewardPool {
    pub total_staked: u64,              // Total amount of tokens staked in the pool
    pub reward_rate: u64,               // Reward rate (e.g., tokens rewarded per second)
    pub last_update_time: u64,          // Timestamp of the last reward rate update
    pub accumulated_reward_per_share: u64, // Accumulated reward per share (used for calculating rewards)
}

// -------------------------------------
// Proposal Structure
// -------------------------------------
#[account]
pub struct Proposal {
    pub description: String,            // The text description of the proposal
    pub new_collateral_ratio: Option<u64>, // Proposed new collateral ratio
    pub new_reward_rate: Option<u64>,   // Proposed new reward rate
    pub approval_votes: u32,            // Number of votes in favor
    pub reject_votes: u32,              // Number of votes against
    pub status: ProposalStatus,         // Current status (Pending, Approved, Rejected)
    pub proposer: Pubkey,               // Address of the proposer
    pub voting_period_end: u64,         // Timestamp when the voting period ends
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}

// -------------------------------------
// Collateral Type Structure
// -------------------------------------
#[account]
pub struct CollateralType {
    pub collateral_mint: Pubkey,        // The mint address of the collateral (e.g., USDC, SOL)
    pub collateral_ratio: u64,          // The required collateral ratio for this type
    pub price_feed: Pubkey,             // Address of the price feed account
    pub liquidation_threshold: u64,     // The threshold below which liquidation can occur
    pub stability_fee: u64,             // Stability fee or interest rate for borrowing against this collateral
}

// -------------------------------------
// System State Structure
// -------------------------------------
#[account]
pub struct SystemState {
    pub staking_paused: bool,           // Indicates if staking is currently paused
    pub governance_authority: Pubkey,   // The current governance authority for the protocol
    pub global_stability_fee: u64,      // Global stability fee for borrowing
    pub minting_fee_rate: u64,          // Fee rate applied when minting stablecoins
}

// -------------------------------------
// Contexts for Instructions
// -------------------------------------

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 8)]
    pub governance: Account<'info, Governance>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintStablecoin<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub stablecoin_mint: Account<'info, Mint>,
    #[account(mut)]
    pub treasury_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub payer: Signer<'info>,
    pub optional_authority: Option<Signer<'info>>,

}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub liquidator_collateral_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staker_account: Account<'info, StakerAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    pub staker_account: Account<'info, StakerAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub staker_account: Account<'info, StakerAccount>,
    #[account(mut)]
    pub user_reward_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_token_mint: Account<'info, Mint>,
    pub reward_mint_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(init, payer = proposer, space = 8 + 200 + 32 + 4 + 4 + 1 + 32)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    #[account(mut)] // Make sure the proposer is mutable since it is paying for the account creation
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddCollateralType<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8 + 32)]
    pub collateral_type: Account<'info, CollateralType>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintStablecoinWithCollateral<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub stablecoin_mint: Account<'info, Mint>,
    #[account(mut)]
    pub collateral_type: Account<'info, CollateralType>,
    pub token_program: Program<'info, Token>,
    pub payer: Signer<'info>,
    pub optional_authority: Option<Signer<'info>>,

}
