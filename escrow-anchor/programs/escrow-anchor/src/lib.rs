pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use error::{ErrorCode};

declare_id!("AFcqjGZoqEUfYY55jjRVvkby43KZqM1ah5TKkUNP7oUq");

#[program]
pub mod escrow_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn check(ctx: Context<Check>) -> Result<()> {
        check::handler(ctx)
    }

    pub fn create(ctx: Context<Create>, maker_amt : u64, taker_amt: u64) -> Result<()> {
        create::handler(ctx, maker_amt, taker_amt)
    }

    pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
        let amount = ctx.accounts.user_b_details.mint_amt;
        deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        withdraw::handler(ctx)
    }

    pub fn close(ctx: Context<Close>) -> Result<()>{
        require!(ctx.accounts.escrow_token_acc_a.amount <= 0, ErrorCode::AccountContainsFund);
        require!(ctx.accounts.escrow_token_acc_b.amount <= 0, ErrorCode::AccountContainsFund);
        close::handler(ctx)
    }
}
