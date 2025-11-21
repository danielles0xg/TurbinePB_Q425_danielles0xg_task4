use anchor_lang::prelude::*;

mod error;
use error::ErrorCode;


declare_id!("EkAeaMi5vj8HEJ1wSc2cmeiFMqcWFgngp1J6F3wF6gq1");



#[program]
pub mod perpetual_nft_lending {
    use super::*;

    pub fn init_lending_market(
        ctx: Context<InitLendingMarketCtx>,
        fee_recipient: Pubkey,
        lender_fee_bps: u64,
        borrower_fee_bps: u64
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
}

#[account]
#[derive(InitSpace)]
pub struct LendingMarket {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub lender_fee_bps: u64, // when getting repaid
    pub borrower_fee_bps: u64, // when taking a loan offer
    pub bump: u8
}

#[derive(Accounts)]
pub struct InitLendingMarketCtx<'info>{
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = LendingMarket::DISCRIMINATOR.len() + LendingMarket::INIT_SPACE,
        seeds = [b"lending_market"],
        bump,
    )]
    pub lending_market: Account<'info, LendingMarket>,
    pub system_program: Program<'info, System>,
}
