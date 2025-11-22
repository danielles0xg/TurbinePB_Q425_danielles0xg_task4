use anchor_lang::prelude::*;
use crate::instructions::init_lending_market::LendingMarket;
use crate::error::ErrorCode;

#[account]
#[derive(InitSpace)]
pub struct AssetPairMarket {
    pub loan_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub is_active: bool,
    pub bump: u8,
}

/// seeds = [AssetPairMarket::SEED, loan_mint.key().as_ref(), collateral_mint.key().as_ref()]
impl AssetPairMarket {
    pub const SEED: &'static [u8] = b"asset_pair";
}


#[derive(Accounts)]
pub struct CreateAssetPairMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin @ ErrorCode::Unauthorized,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    #[account(
        init,
        payer = admin,
        space = 8 + AssetPairMarket::INIT_SPACE,
        seeds = [
            AssetPairMarket::SEED,
            loan_mint.key().as_ref(),
            collateral_mint.key().as_ref()
        ],
        bump,
    )]
    pub asset_pair_market: Account<'info, AssetPairMarket>,

    /// CHECK: Validated as mint in handler
    pub loan_mint: AccountInfo<'info>,

    /// CHECK: Validated as mint in handler
    pub collateral_mint: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_asset_pair_market_handler(ctx: Context<CreateAssetPairMarket>) -> Result<()> {
    let asset_pair_market = &mut ctx.accounts.asset_pair_market;

    asset_pair_market.loan_mint = ctx.accounts.loan_mint.key();
    asset_pair_market.collateral_mint = ctx.accounts.collateral_mint.key();
    asset_pair_market.is_active = true;
    asset_pair_market.bump = ctx.bumps.asset_pair_market;

    Ok(())
}