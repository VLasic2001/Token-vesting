use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
};
use pinocchio_token::{instructions::CloseAccount, state::TokenAccount};

use crate::{AssociatedTokenAccount, ProgramAccount, SignerAccount, VestingSchedule};

pub struct CloseScheduleAccounts<'a> {
    pub authority: &'a AccountInfo,
    pub schedule: &'a AccountInfo,
    pub schedule_ata: &'a AccountInfo,
    pub mint: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CloseScheduleAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, schedule, schedule_ata, mint, token_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;

        Ok(Self {
            authority,
            schedule,
            schedule_ata,
            mint,
            token_program,
        })
    }
}

pub struct CloseSchedule<'a> {
    pub accounts: CloseScheduleAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CloseSchedule<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: CloseScheduleAccounts::try_from(accounts)?,
        })
    }
}

impl<'a> CloseSchedule<'a> {
    pub const DISCRIMINATOR: &'a u8 = &4;

    pub fn process(&self) -> Result<(), ProgramError> {
        AssociatedTokenAccount::check(
            self.accounts.schedule_ata,
            self.accounts.schedule,
            self.accounts.mint,
            self.accounts.token_program,
        )?;

        let ata = TokenAccount::from_account_info(&self.accounts.schedule_ata)?;
        if ata.amount() > 0 {
            return Err(ProgramError::Custom(2));
        }

        let schedule = VestingSchedule::load(self.accounts.schedule)?;
        if self.accounts.authority.key() != schedule.authority() {
            return Err(ProgramError::InvalidAccountData);
        }

        let seed_binding = schedule.seed().to_le_bytes();
        let bump_binding = schedule.schedule_bump();
        let schedule_signer_seeds = [
            Seed::from(b"vesting"),
            Seed::from(&seed_binding),
            Seed::from(schedule.mint()),
            Seed::from(schedule.authority()),
            Seed::from(&bump_binding),
        ];

        let schedule_signer = [Signer::from(&schedule_signer_seeds)];

        drop(ata);
        CloseAccount {
            account: self.accounts.schedule_ata,
            destination: self.accounts.authority,
            authority: self.accounts.schedule,
        }
        .invoke_signed(&schedule_signer)?;

        drop(schedule);
        ProgramAccount::close(self.accounts.schedule, self.accounts.authority)?;

        Ok(())
    }
}
