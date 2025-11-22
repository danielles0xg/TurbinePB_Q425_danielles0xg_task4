use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("EkAeaMi5vj8HEJ1wSc2cmeiFMqcWFgngp1J6F3wF6gq1");

#[program]
pub mod lending_market {
    use super::*;

    /// only admin
    pub fn init_lending_market(
        ctx: Context<InitLendingMarket>,
        fee_recipient: Pubkey,
        lender_fee_bps: u64,
        borrower_fee_bps: u64,
    ) -> Result<()> {
        init_lending_market_handler(
            ctx,
            fee_recipient,
            lender_fee_bps,
            borrower_fee_bps,
        )
    }

    /// only admin
    pub fn create_asset_pair_market(ctx: Context<CreateAssetPairMarket>) -> Result<()> {
        create_asset_pair_market_handler(ctx)
    }

    /// lender
    pub fn create_lending_offer(
        ctx: Context<CreateLendingOffer>,
        offer_id: u64,
        loan_amount: u64,
        interest_rate_bps: u64,
        ltv_bps: u64,
    ) -> Result<()> {
        create_lending_offer_handler(
            ctx,
            offer_id,
            loan_amount,
            interest_rate_bps,
            ltv_bps,
        )
    }

    /// lender
    pub fn cancel_lending_offer(ctx: Context<CancelLendingOffer>) -> Result<()> {
        cancel_lending_offer_handler(ctx)
    }

    /// borrower
    pub fn take_loan(ctx: Context<TakeLoan>, collateral_amount: u64) -> Result<()> {
        take_loan_handler(ctx, collateral_amount)
    }

    /// borrower
    pub fn repay_loan(ctx: Context<RepayLoan>) -> Result<()> {
        repay_loan_handler(ctx)
    }

    /// Request repayment with 48-hour notice
    pub fn request_repayment(ctx: Context<RequestRepayment>) -> Result<()> {
        request_repayment_handler(ctx)
    }

    /// Liquidate loan if deadline passed or LTV exceeds threshold
    pub fn liquidate_loan(ctx: Context<LiquidateLoan>, current_ltv_bps: u64) -> Result<()> {
        liquidate_loan_handler(ctx, current_ltv_bps)
    }
}