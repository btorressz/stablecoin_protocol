use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient collateral to mint stablecoin")]
    InsufficientCollateral,
    #[msg("Insufficient stablecoin balance to burn")]
    InsufficientBalance,
    #[msg("Calculation overflow")]
    Overflow,
    #[msg("Not eligible for liquidation")]
    NotEligibleForLiquidation,
    #[msg("Insufficient funds to stake")]
    InsufficientStakingBalance,
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
    #[msg("Unauthorized operation")]
    UnauthorizedOperation,
    #[msg("Staking pool is empty or unavailable")]
    StakingPoolEmpty,
    #[msg("Invalid collateral ratio")]
    InvalidCollateralRatio,
    #[msg("Invalid amount specified")]
    InvalidAmount,
    #[msg("Invalid lock-up period specified")]
    InvalidLockupPeriod,
    #[msg("Lock-up period has not yet ended")]
    LockupPeriodNotOver,
    #[msg("Rewards have already been claimed recently")]
    RewardsAlreadyClaimed,
    #[msg("Description length exceeds the maximum allowed")]
    DescriptionTooLong,
    #[msg("The proposal has already been concluded")]
    ProposalAlreadyConcluded,
    #[msg("Invalid price value specified")]
    InvalidPrice,
    #[msg("The specified collateral type is not recognized")]
    InvalidCollateralType,
    #[msg("The collateral type already exists")]
    CollateralTypeAlreadyExists,
    #[msg("Cannot delete a collateral type that is still in use")]
    CollateralTypeInUse,
    #[msg("You are not eligible to vote on this proposal")]
    IneligibleToVote,
    #[msg("The proposal cannot be created with no changes specified")]
    ProposalNoChangesSpecified,
    #[msg("Access restricted to governance only")]
    RestrictedToGovernance,
    #[msg("Staking is currently paused")]
    StakingPaused,
    #[msg("Insufficient funds in the insurance pool")]
    InsufficientInsurancePoolBalance,
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Account already initialized")]
    AlreadyInitialized,
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    #[msg("Account data is not in the expected format")]
    InvalidAccountData,
    #[msg("Transaction failed due to insufficient funds")]
    InsufficientFunds,
    #[msg("Feature is not supported at the current time")]
    FeatureNotSupported,
    #[msg("Rate limit exceeded, try again later")]
    RateLimitExceeded,
    #[msg("The voting period has already ended")]
    VotingPeriodEnded,
}
