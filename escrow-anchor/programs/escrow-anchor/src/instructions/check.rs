use anchor_lang::prelude::*;
use crate::{DealDetails, UserEscrowDetails};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// This is a temporary function to check deal/escrow details. Ideally this functionality should be made on the frontend which would take 0 fees

#[derive(Accounts)]
pub struct Check<'info> {
    /// CHECK: Only used for derivation checks, no need for type check
    maker: AccountInfo<'info>,

    // taker: AccountInfo<'info>,
    
    #[account(seeds=[b"deal", maker.key().as_ref()], bump=deal_details.deal_details_bump)]
    deal_details : Account<'info, DealDetails>,
}

pub fn handler(ctx: Context<Check>) -> Result<()> {
    msg!("Deal completion : {:?}", ctx.accounts.deal_details.is_fullfilled);
    Ok(())
}
