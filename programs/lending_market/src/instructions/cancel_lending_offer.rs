use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use crate::error::ErrorCode;

#[account]
#[derive(InitSpace)]
pub struct LendingOffer {
    pub lender: Pubkey,
    pub asset_pair_market: Pubkey,
    pub loan_amount: u64,
    pub interest_rate_bps: u64,  // e.g., 500 = 5% APR
    pub ltv_bps: u64,             // e.g., 8000 = 80% LTV
    pub offer_id: u64,
    pub is_active: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl LendingOffer {
    pub const SEED: &'static [u8] = b"lending_offer";
    pub const ESCROW_SEED: &'static [u8] = b"escrow";
}


#[derive(Accounts)]
pub struct CancelLendingOffer<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(
        mut,
        close = lender,
        has_one = lender,
        constraint = lending_offer.is_active @ ErrorCode::OfferNotActive,
    )]
    pub lending_offer: Account<'info, LendingOffer>,

    #[account(
        mut,
        seeds = [LendingOffer::ESCROW_SEED,lending_offer.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lender_token_account.owner == lender.key(),
    )]
    pub lender_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn cancel_lending_offer_handler(ctx: Context<CancelLendingOffer>) -> Result<()> {
    let lending_offer = &ctx.accounts.lending_offer;
    let offer_key = lending_offer.key();

    // Create escrow authority seeds
    let escrow_seeds = &[
        LendingOffer::ESCROW_SEED,
        offer_key.as_ref(),
        &[ctx.bumps.escrow],
    ];
    let signer_seeds = &[&escrow_seeds[..]];

    // Transfer tokens back from escrow to lender
    let cpi_accounts = Transfer {
        from: ctx.accounts.escrow.to_account_info(),
        to: ctx.accounts.lender_token_account.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, ctx.accounts.escrow.amount)?;

    // Close escrow account
    let cpi_accounts = CloseAccount {
        account: ctx.accounts.escrow.to_account_info(),
        destination: ctx.accounts.lender.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::close_account(cpi_ctx)?;

    Ok(())
}