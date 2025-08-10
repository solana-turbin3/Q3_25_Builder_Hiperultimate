use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DealDetails{
    pub deal_details_bump: u8,
    pub escrow_token_controller_bump: u8,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub is_fullfilled : bool
}


#[account]
#[derive(InitSpace)]
pub struct UserEscrowDetails {
    pub mint_amt : u64,
    pub mint : Pubkey,
    pub escrow_token_acc_bump : u8,
    pub user_details_bump: u8,
}