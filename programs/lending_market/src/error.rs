use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Admin only")]
    Unauthorized,

    #[msg("Basis points must be <= 10_000")]
    FeeTooHigh,

    #[msg("Invalid fee recipient")]
    InvalidFeeRecipient,

    #[msg("Offer is not active")]
    OfferNotActive,

    #[msg("Loan is not active")]
    LoanNotActive,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Asset pair market is not active")]
    MarketNotActive,

    #[msg("Invalid asset pair")]
    InvalidAssetPair,

    #[msg("Invalid LTV ratio")]
    InvalidLTV,

    #[msg("LTV exceeds maximum allowed")]
    LTVExceedsMaximum,

    #[msg("Repayment deadline not set")]
    RepaymentDeadlineNotSet,

    #[msg("Repayment deadline not reached")]
    RepaymentDeadlineNotReached,

    #[msg("Cannot liquidate - LTV is healthy")]
    CannotLiquidateHealthyLoan,

    #[msg("Loan has active borrower")]
    LoanHasActiveBorrower,

    #[msg("Invalid collateral amount")]
    InvalidCollateralAmount,

    #[msg("Interest calculation overflow")]
    InterestCalculationOverflow,

    #[msg("Invalid loan amount")]
    InvalidLoanAmount,

    #[msg("Invalid interest rate")]
    InvalidInterestRate,

    #[msg("Offer already taken")]
    OfferAlreadyTaken,
}
