use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Admin only")]
    Unauthorized,

    #[msg("Basis points must be <= 10_000")]
    FeeTooHigh,

    #[msg("Invalid fee recipient")]
    InvalidFeeRecipient,

    #[msg("Listing is not active")]
    ListingNotActive,

    #[msg("Insufficient funds to complete purchase")]
    InsufficientFunds,

    #[msg("Asset does not match listing")]
    InvalidAsset,
}
