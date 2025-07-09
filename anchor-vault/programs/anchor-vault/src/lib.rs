#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("EdkqLU4A5MEiNCGHj9x8WnSNiYFonybGLBGZL1NgZ3Mh");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(context: Context<Initialize>) -> Result<()> {
        context.accounts.initialize(&context.bumps)
    }

    pub fn deposit(context: Context<Payment>, amount: u64) -> Result<()> {
        context.accounts.deposit(amount)
    }

    pub fn withdraw(context: Context<Payment>, amount: u64) -> Result<()> {
        context.accounts.withdraw(amount)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
        seeds = [VaultState::STATE_SEED, user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VaultState::VAULT_SEED, vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.set_inner(VaultState {
            state_bump: bumps.vault_state,
            vault_bump: bumps.vault,
        });

        let required_rent_exempt_balance =
            Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

        let system_program_account_info = self.system_program.to_account_info();

        let transfer_instruction_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let transfer_context =
            CpiContext::new(system_program_account_info, transfer_instruction_accounts);

        transfer(transfer_context, required_rent_exempt_balance)
    }
}

#[derive(Accounts)]
pub struct Payment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [VaultState::STATE_SEED, user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VaultState::VAULT_SEED, vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Payment<'info> {
    fn deposit(&mut self, amount: u64) -> Result<()> {
        let system_program_account_info = self.system_program.to_account_info();

        let transfer_instruction_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let transfer_context =
            CpiContext::new(system_program_account_info, transfer_instruction_accounts);

        transfer(transfer_context, amount)
    }

    fn withdraw(&mut self, amount: u64) -> Result<()> {
        let system_program_account_info = self.system_program.to_account_info();

        let required_rent_exempt_balance =
            Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

        let transfer_instruction_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let vault_signer_seeds = &[
            VaultState::VAULT_SEED,
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let vault_signer = &[&vault_signer_seeds[..]];

        let transfer_context = CpiContext::new_with_signer(
            system_program_account_info,
            transfer_instruction_accounts,
            vault_signer,
        );

        transfer(transfer_context, amount)?;

        require_gte!(self.vault.get_lamports(), required_rent_exempt_balance);

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub state_bump: u8,
    pub vault_bump: u8,
}

impl VaultState {
    pub const STATE_SEED: &[u8] = b"state";
    pub const VAULT_SEED: &[u8] = b"vault";
}
