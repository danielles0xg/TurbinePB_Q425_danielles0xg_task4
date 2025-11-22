use anchor_lang::prelude::*;
use crate::error::ErrorCode;


#[account]
#[derive(InitSpace)]
pub struct LendingMarket {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub lender_fee_bps: u64,    // 200 = 2% fee when lender gets repaid
    pub borrower_fee_bps: u64,  // 100 = 1% fee when borrower takes loan
    pub bump: u8,
}


impl LendingMarket {
    pub const SEED: &'static [u8] = b"lending_market";
}

#[derive(Accounts)]
pub struct InitLendingMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + LendingMarket::INIT_SPACE,
        seeds = [LendingMarket::SEED],
        bump,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    pub system_program: Program<'info, System>,
}

pub fn init_lending_market_handler(
    ctx: Context<InitLendingMarket>,
    fee_recipient: Pubkey,
    lender_fee_bps: u64,
    borrower_fee_bps: u64,
) -> Result<()> {
    require!(lender_fee_bps <= 10000, ErrorCode::FeeTooHigh);
    require!(borrower_fee_bps <= 10000, ErrorCode::FeeTooHigh);

    let lending_market = &mut ctx.accounts.lending_market;
    lending_market.admin = ctx.accounts.admin.key();
    lending_market.fee_recipient = fee_recipient;
    lending_market.lender_fee_bps = lender_fee_bps;
    lending_market.borrower_fee_bps = borrower_fee_bps;
    lending_market.bump = ctx.bumps.lending_market;

    Ok(())
}