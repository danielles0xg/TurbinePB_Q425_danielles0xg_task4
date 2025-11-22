use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use crate::instructions::init_lending_market::LendingMarket;
use crate::state::Loan;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct RepayLoan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    pub lending_market: Account<'info, LendingMarket>,

    #[account(
        mut,
        close = borrower,
        has_one = borrower,
        constraint = loan.is_active @ ErrorCode::LoanNotActive,
    )]
    pub loan: Account<'info, Loan>,

    #[account(
        mut,
        seeds = [Loan::COLLATERAL_SEED,loan.key().as_ref()],
        bump,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = borrower_loan_token_account.owner == borrower.key(),
    )]
    pub borrower_loan_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = borrower_collateral_token_account.owner == borrower.key(),
    )]
    pub borrower_collateral_token_account: Account<'info, TokenAccount>,

    /// CHECK: Validated as lender from loan
    #[account(
        mut,
        constraint = lender.key() == loan.lender,
    )]
    pub lender: AccountInfo<'info>,

    #[account(
        mut,
        constraint = lender_token_account.owner == lender.key(),
    )]
    pub lender_token_account: Account<'info, TokenAccount>,

    /// CHECK: Validated in lending_market
    #[account(
        mut,
        constraint = fee_recipient.key() == lending_market.fee_recipient @ ErrorCode::InvalidFeeRecipient,
    )]
    pub fee_recipient: AccountInfo<'info>,

    #[account(
        mut,
        constraint = fee_recipient_token_account.owner == fee_recipient.key(),
    )]
    pub fee_recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn repay_loan_handler(ctx: Context<RepayLoan>) -> Result<()> {
    let loan = &ctx.accounts.loan;
    let lending_market = &ctx.accounts.lending_market;
    let current_time = Clock::get()?.unix_timestamp;

    // Calculate total repayment amount (principal + interest)
    let total_repayment = loan.calculate_repayment_amount(current_time)?;

    // Calculate lender fee (2%)
    let lender_fee = total_repayment
        .checked_mul(lending_market.lender_fee_bps)
        .ok_or(ErrorCode::InterestCalculationOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::InterestCalculationOverflow)?;

    let lender_receives = total_repayment
        .checked_sub(lender_fee)
        .ok_or(ErrorCode::InterestCalculationOverflow)?;

    // 1. Transfer repayment amount (minus fee) from borrower to lender
    let cpi_accounts = Transfer {
        from: ctx.accounts.borrower_loan_token_account.to_account_info(),
        to: ctx.accounts.lender_token_account.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, lender_receives)?;

    // 2. Transfer fee from borrower to fee recipient
    let cpi_accounts = Transfer {
        from: ctx.accounts.borrower_loan_token_account.to_account_info(),
        to: ctx.accounts.fee_recipient_token_account.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, lender_fee)?;

    let loan_key = loan.key();

    // Create collateral vault authority seeds
    let collateral_seeds = &[
        Loan::COLLATERAL_SEED,
        loan_key.as_ref(),
        &[ctx.bumps.collateral_vault],
    ];
    let signer_seeds = &[&collateral_seeds[..]];

    // 3. Return collateral from vault to borrower
    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.borrower_collateral_token_account.to_account_info(),
        authority: ctx.accounts.collateral_vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, ctx.accounts.collateral_vault.amount)?;

    // Close collateral vault account
    let cpi_accounts = CloseAccount {
        account: ctx.accounts.collateral_vault.to_account_info(),
        destination: ctx.accounts.borrower.to_account_info(),
        authority: ctx.accounts.collateral_vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::close_account(cpi_ctx)?;

    Ok(())
}