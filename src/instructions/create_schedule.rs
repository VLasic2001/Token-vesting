use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::Seed, program_error::ProgramError,
};

use crate::{AssociatedTokenAccount, ProgramAccount, SignerAccount, VestingSchedule};

pub struct CreateScheduleAccounts<'a> {
    pub creator: &'a AccountInfo,
    pub mint: &'a AccountInfo,
    pub schedule: &'a AccountInfo,
    pub schedule_ata: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,
}
impl<'a> TryFrom<&'a [AccountInfo]> for CreateScheduleAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [
            creator,
            mint,
            schedule,
            schedule_ata,
            system_program,
            token_program,
            associated_token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(creator)?;

        Ok(Self {
            creator,
            mint,
            schedule,
            schedule_ata,
            system_program,
            token_program,
            associated_token_program,
        })
    }
}

#[repr(C, packed)]
pub struct CreateScheduleInstructionData {
    pub authority: [u8; 32],
    pub mint: [u8; 32],
    pub start_time: i64,
    pub cliff_time: i64,
    pub step_duration: i64,
    pub total_vesting_time: i64,
    pub seed: u64,
    pub schedule_bump: [u8; 1],
}

impl<'a> TryFrom<&[u8]> for CreateScheduleInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        const LEN: usize = size_of::<CreateScheduleInstructionData>();

        if data.len() != LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(unsafe { (data.as_ptr() as *const Self).read_unaligned() })
    }
}

pub struct CreateSchedule<'a> {
    pub accounts: CreateScheduleAccounts<'a>,
    pub instruction_data: CreateScheduleInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for CreateSchedule<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = CreateScheduleAccounts::try_from(accounts)?;
        let instruction_data: CreateScheduleInstructionData =
            CreateScheduleInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> CreateSchedule<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        if *self.accounts.creator.key() != self.instruction_data.authority {
            return Err(ProgramError::InvalidInstructionData);
        }
        if *self.accounts.mint.key() != self.instruction_data.mint {
            return Err(ProgramError::InvalidInstructionData);
        }

        if self.instruction_data.step_duration <= 0
            || self.instruction_data.total_vesting_time <= 0
            || self.instruction_data.total_vesting_time < self.instruction_data.cliff_time
            || (self.instruction_data.total_vesting_time - self.instruction_data.cliff_time)
                < self.instruction_data.step_duration
            || self.instruction_data.cliff_time < 0
        {
            return Err(ProgramError::InvalidInstructionData);
        }

        let seed_binding = self.instruction_data.seed.to_le_bytes();
        let schedule_seeds = [
            Seed::from(b"vesting"),
            Seed::from(&seed_binding),
            Seed::from(&self.instruction_data.mint),
            Seed::from(&self.instruction_data.authority),
            Seed::from(&self.instruction_data.schedule_bump),
        ];

        ProgramAccount::init::<VestingSchedule>(
            self.accounts.creator,
            self.accounts.schedule,
            &schedule_seeds,
            VestingSchedule::LEN,
        )?;

        let schedule = unsafe { VestingSchedule::load_mut_unchecked(&self.accounts.schedule)? };
        schedule.set_inner(
            self.instruction_data.authority,
            self.instruction_data.mint,
            self.instruction_data.start_time,
            self.instruction_data.cliff_time,
            self.instruction_data.step_duration,
            self.instruction_data.total_vesting_time,
            self.instruction_data.seed,
            self.instruction_data.schedule_bump,
        )?;

        AssociatedTokenAccount::init(
            self.accounts.schedule_ata,
            self.accounts.mint,
            self.accounts.creator,
            self.accounts.schedule,
            self.accounts.system_program,
            self.accounts.token_program,
        )?;

        Ok(())
    }
}
