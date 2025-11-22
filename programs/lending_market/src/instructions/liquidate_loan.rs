use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use crate::state::Loan;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct LiquidateLoan<'info> {
    pub lender: Signer<'info>,

    #[account(
        mut,
        close = lender,
        has_one = lender,
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
        constraint = lender_token_account.owner == lender.key(),
    )]
    pub lender_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn liquidate_loan_handler(ctx: Context<LiquidateLoan>, current_ltv_bps: u64) -> Result<()> {
    let loan = &ctx.accounts.loan;
    let current_time = Clock::get()?.unix_timestamp;

    // Verify loan can be liquidated
    require!(
        loan.can_liquidate(current_time, current_ltv_bps),
        ErrorCode::CannotLiquidateHealthyLoan
    );

    let loan_key = loan.key();

    // Create collateral vault authority seeds
    let collateral_seeds = &[
        Loan::COLLATERAL_SEED,
        loan_key.as_ref(),
        &[ctx.bumps.collateral_vault],
    ];
    let signer_seeds = &[&collateral_seeds[..]];

    // Transfer all collateral to lender
    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.lender_token_account.to_account_info(),
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
        destination: ctx.accounts.lender.to_account_info(),
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