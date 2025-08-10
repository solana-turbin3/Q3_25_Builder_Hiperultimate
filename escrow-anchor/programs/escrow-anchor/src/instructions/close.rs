use anchor_lang::{ prelude::*};
use crate::{DealDetails, UserEscrowDetails};
use anchor_spl::{ token_2022::{close_account, CloseAccount}, token_interface::{TokenAccount, TokenInterface}};

// This is a temporary function to check deal/escrow details. Ideally this functionality should be made on the frontend which would take 0 fees

#[derive(Accounts)]
pub struct Close<'info> {

    /// CHECK: Public key representing the deal maker
    #[account(mut)]
    maker: Signer<'info>,

    /// CHECK: Public key representing the deal maker
    #[account()]
    taker: AccountInfo<'info>,

    #[account(mut, seeds=[b"deal", maker.key().as_ref()], bump=deal_details.deal_details_bump, close=maker)]
    pub deal_details: Account<'info, DealDetails>,

    pub token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        seeds=[b"controller", maker.key().as_ref()], 
        bump=deal_details.escrow_token_controller_bump,
    )]
    pub escrow_token_controller: SystemAccount<'info>,

    // need to structure deal_details better to identify whose bump this is
    #[account(mut, seeds=[b"token", maker.key().as_ref()], bump=user_a_details.escrow_token_acc_bump)]
    pub escrow_token_acc_a: InterfaceAccount<'info, TokenAccount>,

    #[account(mut, seeds=[b"user_details", maker.key().as_ref()], bump=user_a_details.user_details_bump, close=maker)]
    pub user_a_details: Account<'info, UserEscrowDetails>,

    #[account(mut, seeds=[b"token", taker.key().as_ref()], bump=user_b_details.escrow_token_acc_bump)]
    pub escrow_token_acc_b: InterfaceAccount<'info, TokenAccount>,

    #[account(mut, seeds=[b"user_details", taker.key().as_ref()], bump=user_b_details.user_details_bump, close=maker)]
    pub user_b_details: Account<'info, UserEscrowDetails>,

    system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Close>) -> Result<()> {
    // Closing escrow_a escrow_b and controller through CPI because they are token accounts
    msg!("Deal completion : {:?}", ctx.accounts.deal_details.is_fullfilled);

    let controller_seeds: &[&[&[u8]]] = &[&[b"controller", ctx.accounts.maker.key.as_ref(), &[ctx.accounts.deal_details.escrow_token_controller_bump]]]; 

    // Close escrow A
    close_token_account(
        &ctx.accounts.escrow_token_acc_a,
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.escrow_token_controller.to_account_info(),
        &ctx.accounts.token_program,
        controller_seeds,
    )?;

    // Close escrow B
    close_token_account(
        &ctx.accounts.escrow_token_acc_b,
        &ctx.accounts.maker.to_account_info(),
        &ctx.accounts.escrow_token_controller.to_account_info(),
        &ctx.accounts.token_program,
        controller_seeds,
    )?;

    let leftover_funds = ctx.accounts.escrow_token_controller.lamports();
    if leftover_funds > 0{

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.escrow_token_controller.to_account_info(),
            to: ctx.accounts.maker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            cpi_accounts,
            controller_seeds,
        );

        // transfer(tx, leftover_funds)?
        anchor_lang::system_program::transfer(cpi_ctx, leftover_funds)?;
    }


    Ok(())
}

fn close_token_account<'info>(
    token_acc: &InterfaceAccount<'info, TokenAccount>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &Interface<'info, TokenInterface>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_accounts = CloseAccount {
        account: token_acc.to_account_info(),
        destination: destination.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    close_account(cpi_ctx)
}