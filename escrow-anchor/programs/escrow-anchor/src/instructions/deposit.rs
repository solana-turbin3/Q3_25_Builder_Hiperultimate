use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{DealDetails, UserEscrowDetails};

#[derive(Accounts)]
pub struct Deposit<'info> {
    // #[account(mut)]
    // pub signer: Signer<'info>,
    /// CHECK: just used as a public key wallet, doesnt need validation
    pub maker: AccountInfo<'info>,

    /// CHECK: just used as a public key wallet, doesnt need validation
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut, seeds=[b"deal", maker.key().as_ref()], bump=deal_details.deal_details_bump)]
    pub deal_details: Account<'info, DealDetails>,

    #[account(mut,
        associated_token::mint = user_b_details.mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub user_token_acc_b: InterfaceAccount<'info, TokenAccount>,

    #[account(mint::token_program=token_program)]
    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

    // need to structure deal_details better to identify whose bump this is
    #[account(seeds=[b"token", maker.key().as_ref()], bump=user_a_details.escrow_token_acc_bump)]
    pub escrow_token_acc_a: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds=[b"user_details", maker.key().as_ref()], bump=user_a_details.user_details_bump)]
    pub user_a_details: Account<'info, UserEscrowDetails>,

    #[account(mut, seeds=[b"token", taker.key().as_ref()], bump=user_b_details.escrow_token_acc_bump)]
    pub escrow_token_acc_b: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds=[b"user_details", taker.key().as_ref()], bump=user_b_details.user_details_bump)]
    pub user_b_details: Account<'info, UserEscrowDetails>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    msg!("Deposit initiating of amount: {:?}", amount);

    // Check if signer has already submitted required tokens
    // let deal_details = ctx.accounts.deal_details.to_account_info();
    // if deal_details.

    let decimals = ctx.accounts.mint.decimals;

    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.user_token_acc_b.to_account_info(),
        to: ctx.accounts.escrow_token_acc_b.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

    transfer_checked(cpi_context, amount, decimals)?;

    // After sending token we update the current value
    ctx.accounts.escrow_token_acc_b.reload()?;

    // Probably very risky here to check user_b_details.mint_amt, need to add better seeds to make it secure. Atm im just learning, we can add ids to make seeds secure
    // Check if both sides has transfered their amount of tokens, if yes transfer
    if ctx.accounts.user_a_details.mint_amt <= ctx.accounts.escrow_token_acc_a.amount
        && ctx.accounts.user_b_details.mint_amt <= ctx.accounts.escrow_token_acc_b.amount
    {
        msg!("WE ARE RUNNING");
        // let the users withdraw
        ctx.accounts.deal_details.is_fullfilled = true;
    }
    Ok(())
}
