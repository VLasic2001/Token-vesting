use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_token::instructions::Transfer;

use crate::{ProgramAccount, SignerAccount, VestingAllocation, VestingSchedule};

pub struct WithdrawAccounts<'a> {
    pub recipient: &'a AccountInfo,
    pub recipient_ata: &'a AccountInfo,
    pub authority: &'a AccountInfo,
    pub schedule: &'a AccountInfo,
    pub schedule_ata: &'a AccountInfo,
    pub allocation: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [
            recipient,
            recipient_ata,
            authority,
            schedule,
            schedule_ata,
            allocation,
            token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(recipient)?;

        Ok(Self {
            recipient,
            recipient_ata,
            authority,
            schedule,
            schedule_ata,
            allocation,
            token_program,
        })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: WithdrawAccounts::try_from(accounts)?,
        })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(&self) -> Result<(), ProgramError> {
        let mut allocation = VestingAllocation::load_mut(self.accounts.allocation)?;

        if self.accounts.recipient.key() != allocation.recipient() {
            return Err(ProgramError::InvalidAccountData);
        }
        let (allocation_pda, _) = find_program_address(
            &[
                &Seed::from(b"allocation"),
                &Seed::from(self.accounts.recipient.key()),
                &Seed::from(self.accounts.schedule.key()),
            ],
            &crate::ID,
        );

        if allocation_pda.ne(self.accounts.allocation.key()) {
            return Err(ProgramError::InvalidAccountData);
        }

        let schedule = VestingSchedule::load(self.accounts.schedule)?;

        if self.accounts.authority.key() != schedule.authority() {
            return Err(ProgramError::InvalidAccountData);
        }

        if schedule.start_time() + schedule.cliff_time() > Clock::get()?.unix_timestamp as i64 {
            return Err(ProgramError::Custom(0));
        }

        let time_elapsed =
            Clock::get()?.unix_timestamp as i64 - schedule.start_time() - schedule.cliff_time();
        let steps_passed = time_elapsed / schedule.step_duration();
        let total_steps =
            (schedule.total_vesting_time() - schedule.cliff_time()) / schedule.step_duration();

        if steps_passed == 0 {
            return Err(ProgramError::Custom(1));
        }

        let withdrawable_amount: u64;
        if steps_passed >= total_steps {
            withdrawable_amount = allocation.vesting_total() - allocation.withdrawn_amount();
        } else {
            let amount_to_vest_per_step = allocation.vesting_total() / total_steps as u64;
            let vested_amount = steps_passed as u64 * amount_to_vest_per_step;
            withdrawable_amount = vested_amount.saturating_sub(allocation.withdrawn_amount());
        }

        if withdrawable_amount == 0 {
            return Err(ProgramError::Custom(1));
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

        Transfer {
            amount: withdrawable_amount,
            from: self.accounts.schedule_ata,
            to: self.accounts.recipient_ata,
            authority: self.accounts.schedule,
        }
        .invoke_signed(&schedule_signer)?;

        let new_withdrawn_amount = allocation.withdrawn_amount() + withdrawable_amount;
        let vesting_total = allocation.vesting_total();
        allocation.set_withdrawn_amount(new_withdrawn_amount)?;

        if new_withdrawn_amount == vesting_total {
            drop(allocation);
            ProgramAccount::close(self.accounts.allocation, self.accounts.authority)?;
        }

        Ok(())
    }
}
