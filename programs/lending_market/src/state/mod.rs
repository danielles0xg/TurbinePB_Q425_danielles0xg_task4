use anchor_lang::prelude::*;
use crate::error::ErrorCode;

#[account]
#[derive(InitSpace)]
pub struct Loan {
    pub lending_offer: Pubkey,
    pub lender: Pubkey,
    pub borrower: Pubkey,
    pub principal_amount: u64,
    pub collateral_amount: u64,
    pub interest_rate_bps: u64,   // locked from offer
    pub ltv_bps: u64,              // locked from offer
    pub loan_start_time: i64,
    pub last_interest_update: i64,
    #[max_len(1)]
    pub repayment_deadline: Option<i64>,  // 48hr notice
    pub is_active: bool,
    pub bump: u8,
}

impl Loan {
    pub const SEED: &'static [u8] = b"loan";
    pub const COLLATERAL_SEED: &'static [u8] = b"collateral";
    pub const REPAYMENT_NOTICE_DURATION: i64 = 48 * 60 * 60; // 48 hours in seconds

    /// Calculate the current interest owed
    pub fn calculate_interest(&self, current_time: i64) -> Result<u64> {
        let time_elapsed = current_time.saturating_sub(self.loan_start_time);
        let days_elapsed = time_elapsed / 86400; // Convert seconds to days

        // interest = principal * (rate_bps/10000) * (days_elapsed/365)
        let interest = (self.principal_amount as u128)
            .checked_mul(self.interest_rate_bps as u128)
            .ok_or(error!(ErrorCode::InterestCalculationOverflow))?
            .checked_mul(days_elapsed as u128)
            .ok_or(error!(ErrorCode::InterestCalculationOverflow))?
            .checked_div(10000 * 365)
            .ok_or(error!(ErrorCode::InterestCalculationOverflow))?;

        Ok(interest as u64)
    }

    /// Calculate total repayment amount (principal + interest)
    pub fn calculate_repayment_amount(&self, current_time: i64) -> Result<u64> {
        let interest = self.calculate_interest(current_time)?;
        self.principal_amount
            .checked_add(interest)
            .ok_or(error!(ErrorCode::InterestCalculationOverflow))
    }

    /// Check if loan can be liquidated
    pub fn can_liquidate(&self, current_time: i64, current_ltv_bps: u64) -> bool {
        // Can liquidate if:
        // 1. Repayment deadline has passed
        if let Some(deadline) = self.repayment_deadline {
            if current_time > deadline {
                return true;
            }
        }

        // 2. LTV exceeds 120% (12000 bps)
        current_ltv_bps > 12000
    }
}