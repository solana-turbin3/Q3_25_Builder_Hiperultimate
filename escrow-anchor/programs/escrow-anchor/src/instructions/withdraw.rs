use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::DealDetails;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub signer: Signer<'info>,

    /// CHECK: just used as a public key wallet, doesnt need validation
    pub maker: AccountInfo<'info>,

    /// CHECK: just used as a public key wallet, doesnt need validation
    pub taker: AccountInfo<'info>,

    #[account(
        seeds=[b"deal", maker.key().as_ref()], 
        bump=deal_details.deal_details_bump
    )]
    pub deal_details: Account<'info, DealDetails>,

    #[account(
        seeds=[b"controller", maker.key().as_ref()], 
        bump=deal_details.escrow_token_controller_bump
    )]
    pub escrow_token_controller: SystemAccount<'info>,

    #[account(
        mut,
        seeds=[b"token", maker.key().as_ref()],
        token::mint=mint_a,
        token::authority=escrow_token_controller,
        bump
    )]
    pub escrow_token_acc_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds=[b"token", taker.key().as_ref()],
        token::mint=mint_b,
        token::authority=escrow_token_controller,
        bump
    )]
    pub escrow_token_acc_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint=mint_exchange,
        token::authority=signer,
        token::token_program=token_program
    )]
    pub user_token_acc: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mint::token_program=token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,
    
    #[account(mint::token_program=token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,
    
    #[account(mint::token_program=token_program)]
    pub mint_exchange: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<Withdraw>) -> Result<()> {
    msg!("Withdrawing from: {:?}", ctx.program_id);

    // return if is_fullfilled is false
    require!(
        ctx.accounts.deal_details.is_fullfilled == true,
        ErrorCode::IncompleteDeal
    );

    require!(
        ctx.accounts.signer.key() == ctx.accounts.deal_details.maker || ctx.accounts.signer.key() == ctx.accounts.deal_details.taker,
        ErrorCode::InvalidUser
    );


    let from_escrow_account;
    if ctx.accounts.signer.key() == ctx.accounts.deal_details.maker.key() {
        from_escrow_account = &ctx.accounts.escrow_token_acc_b;
    }else{
        from_escrow_account = &ctx.accounts.escrow_token_acc_a;
    }

    // let mint_decimals = ctx.accounts.mint.decimals;
    let total_holding_amount = from_escrow_account.amount;

    let controller_seeds: &[&[&[u8]]] = &[&[
        b"controller",
        ctx.accounts.maker.key.as_ref(),
        &[ctx.accounts.deal_details.escrow_token_controller_bump],
    ]];

    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mint_exchange.to_account_info(),
        from: from_escrow_account.to_account_info(),
        to: ctx.accounts.user_token_acc.to_account_info(),
        authority: ctx.accounts.escrow_token_controller.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(controller_seeds);
    transfer_checked(cpi_context, total_holding_amount, ctx.accounts.mint_exchange.decimals)?;
    Ok(())
}