use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::Seed, program_error::ProgramError,
};
use pinocchio_token::instructions::Transfer;

use crate::{
    ProgramAccount, SignerAccount, SystemProgram, TokenProgram, VestingAllocation, VestingSchedule,
};

pub struct CreateAllocationAccounts<'a> {
    pub creator: &'a AccountInfo,
    pub creator_ata: &'a AccountInfo,
    pub schedule: &'a AccountInfo,
    pub schedule_ata: &'a AccountInfo,
    pub allocation: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CreateAllocationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [
            creator,
            creator_ata,
            schedule,
            schedule_ata,
            allocation,
            system_program,
            token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(creator)?;
        SystemProgram::check(system_program)?;
        TokenProgram::check(token_program)?;

        Ok(Self {
            creator,
            creator_ata,
            schedule,
            schedule_ata,
            allocation,
            system_program,
            token_program,
        })
    }
}

#[repr(C, packed)]
pub struct CreateAllocationInstructionData {
    pub recipient: [u8; 32],
    pub amount: u64,
    pub allocation_bump: [u8; 1],
}

impl TryFrom<&[u8]> for CreateAllocationInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        const LEN: usize = size_of::<CreateAllocationInstructionData>();
        if data.len() != LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let recipient = data[0..32].try_into().unwrap();
        let amount = u64::from_le_bytes(data[32..40].try_into().unwrap());
        let allocation_bump = data[40..41].try_into().unwrap();

        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            recipient,
            amount,
            allocation_bump,
        })
    }
}

pub struct CreateAllocation<'a> {
    pub accounts: CreateAllocationAccounts<'a>,
    pub instruction_data: CreateAllocationInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for CreateAllocation<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: CreateAllocationAccounts::try_from(accounts)?,
            instruction_data: CreateAllocationInstructionData::try_from(data)?,
        })
    }
}

impl<'a> CreateAllocation<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&self) -> ProgramResult {
        let schedule = VestingSchedule::load(self.accounts.schedule)?;

        if self.accounts.creator.key() != schedule.authority() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let seed_binding = self.instruction_data.allocation_bump;
        let allocation_seeds = [
            Seed::from(b"allocation"),
            Seed::from(&self.instruction_data.recipient),
            Seed::from(self.accounts.schedule.key()),
            Seed::from(&seed_binding),
        ];

        ProgramAccount::init::<VestingAllocation>(
            self.accounts.creator,
            self.accounts.allocation,
            &allocation_seeds,
            VestingAllocation::LEN,
        )?;

        let allocation =
            unsafe { VestingAllocation::load_mut_unchecked(self.accounts.allocation)? };
        allocation.set_inner(
            self.instruction_data.recipient,
            self.instruction_data.amount,
            0,
        )?;

        Transfer {
            amount: self.instruction_data.amount,
            from: self.accounts.creator_ata,
            to: self.accounts.schedule_ata,
            authority: self.accounts.creator,
        }
        .invoke()?;

        Ok(())
    }
}
