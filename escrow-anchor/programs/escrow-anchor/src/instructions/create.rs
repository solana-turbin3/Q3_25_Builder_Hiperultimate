use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, 
        token_interface::{
        transfer_checked,
        TransferChecked, 
        Mint,
        TokenAccount,
        TokenInterface
    }
};

use crate::{DealDetails,UserEscrowDetails};

// Instruction to create the deal
// need to make it more optimized by somehow storing the bumps
#[derive(Accounts)]
pub struct Create<'info> {
    // to store who made it, the maker will only be able to delete it
    #[account(mut)]
    pub maker : Signer<'info>, 

    /// CHECK: just used as a public key wallet, doesnt need validation
    pub taker: AccountInfo<'info>,

    // stores a unique identifier for this specific deal and the amount both users are supposed to pay. Also stores bumps
    #[account(init, payer=maker, seeds=[b"deal", maker.key().as_ref()], space=8+DealDetails::INIT_SPACE, bump)] // We may need to update the seeds to actually let users make multiple deals
    pub deal_details : Account<'info, DealDetails>, 

    // mint account for both tokens
    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,
    
    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // token program can only be of token_2022
    pub token_program : Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    // existing user token account which they are transferring the fund from
    #[account(
        init_if_needed,
        payer=maker,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub user_token_acc_a: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds=[b"controller", maker.key().as_ref()], bump) ]
    pub escrow_token_controller : SystemAccount<'info>,

    // account program will create to store users token temporarily
    #[account(
        init_if_needed,
        payer=maker, 
        seeds=[b"token", maker.key().as_ref()],
        token::mint = mint_a,
        token::authority = escrow_token_controller,
        token::token_program = token_program, 
        bump)]
    pub escrow_token_acc_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer=maker, 
        seeds=[b"token", taker.key().as_ref()],
        token::mint = mint_b,
        token::authority = escrow_token_controller,
        token::token_program = token_program,
        bump)]
    pub escrow_token_acc_b: InterfaceAccount<'info, TokenAccount>,

    #[account(init, payer=maker, seeds=[b"user_details", maker.key().as_ref() ], bump, space=8+UserEscrowDetails::INIT_SPACE)]
    pub user_a_details : Account<'info, UserEscrowDetails>,

    #[account(init, payer=maker, seeds=[b"user_details", taker.key().as_ref() ], bump, space=8+UserEscrowDetails::INIT_SPACE)]
    pub user_b_details : Account<'info, UserEscrowDetails>,
}

pub fn handler(ctx: Context<Create>, maker_amt : u64, taker_amt: u64) -> Result<()> {
    // Store passed accounts into user_a_details and user_b_details accordingly
    ctx.accounts.deal_details.deal_details_bump = ctx.bumps.deal_details;
    ctx.accounts.deal_details.escrow_token_controller_bump = ctx.bumps.escrow_token_controller;
    ctx.accounts.deal_details.maker = ctx.accounts.maker.key();
    ctx.accounts.deal_details.taker = ctx.accounts.taker.key();
    ctx.accounts.deal_details.is_fullfilled = false;

    // set maker details
    ctx.accounts.user_a_details.mint_amt = maker_amt;
    ctx.accounts.user_a_details.mint = ctx.accounts.mint_a.key();
    // ctx.accounts.user_a_details.user_token_acc = ctx.accounts.user_token_acc_a.key();
    // ctx.accounts.user_a_details.escrow_token_acc = ctx.accounts.escrow_token_acc_a.key();
    ctx.accounts.user_a_details.escrow_token_acc_bump = ctx.bumps.escrow_token_acc_a;
    ctx.accounts.user_a_details.user_details_bump = ctx.bumps.user_a_details;

    // set taker details
    ctx.accounts.user_b_details.mint_amt = taker_amt;
    ctx.accounts.user_b_details.mint = ctx.accounts.mint_b.key();
    // ctx.accounts.user_b_details.user_token_acc = ctx.accounts.user_token_acc_b.key();
    // ctx.accounts.user_b_details.escrow_token_acc = ctx.accounts.escrow_token_acc_b.key();
    ctx.accounts.user_b_details.escrow_token_acc_bump = ctx.bumps.escrow_token_acc_b;
    ctx.accounts.user_b_details.user_details_bump = ctx.bumps.user_b_details;

    // Deposit mint_a from user_token_acc_a to escrow_token_acc_a
    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mint_a.to_account_info(),
        from: ctx.accounts.user_token_acc_a.to_account_info(),
        to: ctx.accounts.escrow_token_acc_a.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

    transfer_checked(cpi_context, maker_amt, ctx.accounts.mint_a.decimals)?;

    msg!("Escrow deal created {:?}", ctx.program_id);
    msg!("Deal created by {:?}", ctx.accounts.maker);
    msg!("Maker amount submitted");

    Ok(())
}
