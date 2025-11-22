use anchor_lang::prelude::*;
use crate::state::Loan;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct RequestRepayment<'info> {
    pub lender: Signer<'info>,

    #[account(
        mut,
        has_one = lender,
        constraint = loan.is_active @ ErrorCode::LoanNotActive,
    )]
    pub loan: Account<'info, Loan>,
}

pub fn request_repayment_handler(ctx: Context<RequestRepayment>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let current_time = Clock::get()?.unix_timestamp;

    // Set repayment deadline to 48 hours from now
    loan.repayment_deadline = Some(current_time + Loan::REPAYMENT_NOTICE_DURATION);

    Ok(())
}