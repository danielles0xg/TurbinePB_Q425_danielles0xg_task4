use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::instructions::create_asset_pair_market::AssetPairMarket;
use crate::instructions::cancel_lending_offer::LendingOffer;
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(offer_id: u64, loan_amount: u64, interest_rate_bps: u64, ltv_bps: u64)]
pub struct CreateLendingOffer<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(
        constraint = asset_pair_market.is_active @ ErrorCode::MarketNotActive,
        constraint = asset_pair_market.loan_mint == loan_mint.key() @ ErrorCode::InvalidAssetPair,
    )]
    pub asset_pair_market: Account<'info, AssetPairMarket>,

    #[account(
        init,
        payer = lender,
        space = 8 + LendingOffer::INIT_SPACE,
        seeds = [
            LendingOffer::SEED,
            lender.key().as_ref(),
            offer_id.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub lending_offer: Account<'info, LendingOffer>,

    #[account(
        init,
        payer = lender,
        token::mint = loan_mint,
        token::authority = escrow,
        seeds = [
            LendingOffer::ESCROW_SEED,
            lending_offer.key().as_ref()
        ],
        bump,
    )]
    pub escrow: Account<'info, TokenAccount>,

    /// CHECK: Validated against asset_pair_market
    pub loan_mint: AccountInfo<'info>,

    #[account(
        mut,
        constraint = lender_token_account.owner == lender.key(),
        constraint = lender_token_account.mint == loan_mint.key(),
    )]
    pub lender_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn create_lending_offer_handler(
    ctx: Context<CreateLendingOffer>,
    offer_id: u64,
    loan_amount: u64,
    interest_rate_bps: u64,
    ltv_bps: u64,
) -> Result<()> {
    require!(loan_amount > 0, ErrorCode::InvalidLoanAmount);
    require!(interest_rate_bps <= 10000, ErrorCode::InvalidInterestRate);
    require!(ltv_bps > 0 && ltv_bps <= 10000, ErrorCode::InvalidLTV);

    // Transfer loan tokens from lender to escrow
    let cpi_accounts = Transfer {
        from: ctx.accounts.lender_token_account.to_account_info(),
        to: ctx.accounts.escrow.to_account_info(),
        authority: ctx.accounts.lender.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, loan_amount)?;

    // Initialize lending offer
    let lending_offer = &mut ctx.accounts.lending_offer;
    lending_offer.lender = ctx.accounts.lender.key();
    lending_offer.asset_pair_market = ctx.accounts.asset_pair_market.key();
    lending_offer.loan_amount = loan_amount;
    lending_offer.interest_rate_bps = interest_rate_bps;
    lending_offer.ltv_bps = ltv_bps;
    lending_offer.offer_id = offer_id;
    lending_offer.is_active = true;
    lending_offer.created_at = Clock::get()?.unix_timestamp;
    lending_offer.bump = ctx.bumps.lending_offer;

    Ok(())
}