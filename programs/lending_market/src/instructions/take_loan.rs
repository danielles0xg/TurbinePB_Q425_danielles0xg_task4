use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::instructions::init_lending_market::LendingMarket;
use crate::instructions::create_asset_pair_market::AssetPairMarket;
use crate::instructions::cancel_lending_offer::LendingOffer;
use crate::state::Loan;
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(collateral_amount: u64)]
pub struct TakeLoan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    pub lending_market: Box<Account<'info, LendingMarket>>,

    #[account(
        constraint = asset_pair_market.loan_mint == loan_mint.key() @ ErrorCode::InvalidAssetPair,
        constraint = asset_pair_market.collateral_mint == collateral_mint.key() @ ErrorCode::InvalidAssetPair,
    )]
    pub asset_pair_market: Box<Account<'info, AssetPairMarket>>,

    #[account(mut,has_one = asset_pair_market,constraint = lending_offer.is_active @ ErrorCode::OfferNotActive)]
    pub lending_offer: Box<Account<'info, LendingOffer>>,

    #[account(
        init,
        payer = borrower,
        space = 8 + Loan::INIT_SPACE,
        seeds = [Loan::SEED,lending_offer.key().as_ref(),borrower.key().as_ref()],
        bump,
    )]
    pub loan: Box<Account<'info, Loan>>,

    #[account(
        mut,
        seeds = [LendingOffer::ESCROW_SEED,lending_offer.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = borrower,
        token::mint = collateral_mint,
        token::authority = collateral_vault,
        seeds = [Loan::COLLATERAL_SEED,loan.key().as_ref()],
        bump,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    /// CHECK: Validated against asset_pair_market
    pub loan_mint: AccountInfo<'info>,

    /// CHECK: Validated against asset_pair_market
    pub collateral_mint: AccountInfo<'info>,

    #[account(
        mut,
        constraint = borrower_loan_token_account.owner == borrower.key(),
        constraint = borrower_loan_token_account.mint == loan_mint.key(),
    )]
    pub borrower_loan_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = borrower_collateral_token_account.owner == borrower.key(),
        constraint = borrower_collateral_token_account.mint == collateral_mint.key(),
    )]
    pub borrower_collateral_token_account: Account<'info, TokenAccount>,

    /// CHECK: Validated in lending_market
    #[account(
        mut,
        constraint = fee_recipient.key() == lending_market.fee_recipient @ ErrorCode::InvalidFeeRecipient,
    )]
    pub fee_recipient: AccountInfo<'info>,

    #[account(
        mut,
        constraint = fee_recipient_token_account.owner == fee_recipient.key(),
        constraint = fee_recipient_token_account.mint == loan_mint.key(),
    )]
    pub fee_recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// 1. Validate collateral amount based on LTV
/// 2. Calculate borrower fee (1%)
/// 3. Transfer collateral from borrower to collateral vault
/// 4. Create escrow authority seeds
/// 5. Transfer loan amount (minus fee) from escrow to borrower
/// 6. Transfer fee from escrow to fee recipient
/// 7. Initialize loan
/// 8. Mark offer as inactive since it's been taken

pub fn take_loan_handler(ctx: Context<TakeLoan>, collateral_amount: u64) -> Result<()> {
    let lending_offer = &ctx.accounts.lending_offer;
    let lending_market = &ctx.accounts.lending_market;
    let loan_amount = lending_offer.loan_amount;

    // Validate collateral amount based on LTV
    // Required collateral = (loan_value / ltv_bps) * 10000
    let required_collateral = (loan_amount as u128)
        .checked_mul(10000)
        .ok_or(ErrorCode::InvalidCollateralAmount)?
        .checked_div(lending_offer.ltv_bps as u128)
        .ok_or(ErrorCode::InvalidCollateralAmount)? as u64;

    require!(
        collateral_amount >= required_collateral,
        ErrorCode::InvalidCollateralAmount
    );

    // Calculate borrower fee (1%)
    let borrower_fee = loan_amount
        .checked_mul(lending_market.borrower_fee_bps)
        .ok_or(ErrorCode::InterestCalculationOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::InterestCalculationOverflow)?;

    let borrower_receives = loan_amount
        .checked_sub(borrower_fee)
        .ok_or(ErrorCode::InterestCalculationOverflow)?;

    // 1. Transfer collateral from borrower to collateral vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.borrower_collateral_token_account.to_account_info(),
        to: ctx.accounts.collateral_vault.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, collateral_amount)?;

    // Create escrow authority seeds
    let lending_offer_key = lending_offer.key();
    let escrow_seeds = &[
        LendingOffer::ESCROW_SEED,
        lending_offer_key.as_ref(),
        &[ctx.bumps.escrow],
    ];
    let signer_seeds = &[&escrow_seeds[..]];

    // 2. Transfer loan amount (minus fee) from escrow to borrower
    let cpi_accounts = Transfer {
        from: ctx.accounts.escrow.to_account_info(),
        to: ctx.accounts.borrower_loan_token_account.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, borrower_receives)?;

    // 3. Transfer fee from escrow to fee recipient
    let cpi_accounts = Transfer {
        from: ctx.accounts.escrow.to_account_info(),
        to: ctx.accounts.fee_recipient_token_account.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, borrower_fee)?;

    { 
        // Initialize loan
        let loan = &mut ctx.accounts.loan;
        let current_time = Clock::get()?.unix_timestamp;

        loan.lending_offer = lending_offer.key();
        loan.lender = lending_offer.lender;
        loan.borrower = ctx.accounts.borrower.key();
        loan.principal_amount = loan_amount;
        loan.collateral_amount = collateral_amount;
        loan.interest_rate_bps = lending_offer.interest_rate_bps;
        loan.ltv_bps = lending_offer.ltv_bps;
        loan.loan_start_time = current_time;
        loan.last_interest_update = current_time;
        loan.repayment_deadline = None;
        loan.is_active = true;
        loan.bump = ctx.bumps.loan;
    }

    // Mark offer as inactive since it's been taken
    let lending_offer = &mut ctx.accounts.lending_offer;
    lending_offer.is_active = false;

    Ok(())
}