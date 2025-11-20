use anchor_lang::prelude::*;

declare_id!("EkAeaMi5vj8HEJ1wSc2cmeiFMqcWFgngp1J6F3wF6gq1");

#[program]
pub mod fractals_market {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
