use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Either both or a single party has yet to pay for their part of the transaction")]
    IncompleteDeal,

    #[msg("User does not have the permission to interact with this deal")]
    InvalidUser,

    #[msg("Unable to clear account, it still contains funds")]
    AccountContainsFund
}