use pinocchio::{
    account_info::{AccountInfo, Ref, RefMut},
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[repr(C)]
pub struct VestingSchedule {
    authority: Pubkey,
    mint: Pubkey,
    start_time: i64,
    cliff_time: i64,
    step_duration: i64,
    total_vesting_time: i64,
    seed: [u8; 8],
    schedule_bump: [u8; 1],
}

impl VestingSchedule {
    pub const LEN: usize = size_of::<VestingSchedule>();

    #[inline(always)]
    pub fn load(account_info: &AccountInfo) -> Result<Ref<'_, Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    #[inline(always)]
    pub unsafe fn load_unchecked(account_info: &AccountInfo) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(unsafe { Self::from_bytes_unchecked(account_info.borrow_data_unchecked()) })
    }

    #[inline(always)]
    pub fn load_mut(account_info: &AccountInfo) -> Result<RefMut<'_, Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(RefMut::map(
            account_info.try_borrow_mut_data()?,
            |data| unsafe { Self::from_bytes_unchecked_mut(data) },
        ))
    }

    /// Load a mutable reference without borrow tracking.
    /// Use after CPI operations where the runtime may leave borrow state corrupted.
    #[inline(always)]
    pub unsafe fn load_mut_unchecked(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(unsafe { Self::from_bytes_unchecked_mut(
            account_info.borrow_mut_data_unchecked(),
        ) })
    }

    #[inline(always)]
    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }
    #[inline(always)]
    pub fn mint(&self) -> &Pubkey {
        &self.mint
    }
    #[inline(always)]
    pub fn start_time(&self) -> i64 {
        self.start_time
    }
    #[inline(always)]
    pub fn cliff_time(&self) -> i64 {
        self.cliff_time
    }
    #[inline(always)]
    pub fn step_duration(&self) -> i64 {
        self.step_duration
    }
    #[inline(always)]
    pub fn total_vesting_time(&self) -> i64 {
        self.total_vesting_time
    }
    #[inline(always)]
    pub fn seed(&self) -> u64 {
        u64::from_le_bytes(self.seed)
    }
    #[inline(always)]
    pub fn schedule_bump(&self) -> [u8; 1] {
        self.schedule_bump
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const VestingSchedule) }
    }
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut VestingSchedule) }
    }

    #[inline(always)]
    pub fn set_authority(&mut self, authority: Pubkey) -> Result<(), ProgramError> {
        self.authority = authority;

        Ok(())
    }
    #[inline(always)]
    pub fn set_mint(&mut self, mint: Pubkey) -> Result<(), ProgramError> {
        self.mint = mint;

        Ok(())
    }
    #[inline(always)]
    pub fn set_start_time(&mut self, start_time: i64) -> Result<(), ProgramError> {
        self.start_time = start_time;

        Ok(())
    }
    #[inline(always)]
    pub fn set_cliff_time(&mut self, cliff_time: i64) -> Result<(), ProgramError> {
        self.cliff_time = cliff_time;

        Ok(())
    }
    #[inline(always)]
    pub fn set_step_duration(&mut self, step_duration: i64) -> Result<(), ProgramError> {
        self.step_duration = step_duration;

        Ok(())
    }
    #[inline(always)]
    pub fn set_total_vesting_time(&mut self, total_vesting_time: i64) -> Result<(), ProgramError> {
        self.total_vesting_time = total_vesting_time;

        Ok(())
    }
    pub fn set_seed(&mut self, seed: u64) -> Result<(), ProgramError> {
        self.seed = seed.to_le_bytes();

        Ok(())
    }
    #[inline(always)]
    pub fn set_schedule_bump(&mut self, schedule_bump: [u8; 1]) -> Result<(), ProgramError> {
        self.schedule_bump = schedule_bump;

        Ok(())
    }

    #[inline(always)]
    pub fn set_inner(
        &mut self,
        authority: Pubkey,
        mint: Pubkey,
        start_time: i64,
        cliff_time: i64,
        step_duration: i64,
        total_vesting_time: i64,
        seed: u64,
        schedule_bump: [u8; 1],
    ) -> Result<(), ProgramError> {
        self.set_authority(authority)?;
        self.set_mint(mint)?;
        self.set_start_time(start_time)?;
        self.set_cliff_time(cliff_time)?;
        self.set_step_duration(step_duration)?;
        self.set_total_vesting_time(total_vesting_time)?;
        self.set_seed(seed)?;
        self.set_schedule_bump(schedule_bump)?;
        Ok(())
    }
}

#[repr(C)]
pub struct VestingAllocation {
    recipient: Pubkey,
    vesting_total: u64,
    withdrawn_amount: u64,
}

impl VestingAllocation {
    pub const LEN: usize = size_of::<VestingAllocation>();

    #[inline(always)]
    pub fn load(account_info: &AccountInfo) -> Result<Ref<'_, Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    #[inline(always)]
    pub unsafe fn load_unchecked(account_info: &AccountInfo) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(unsafe { Self::from_bytes_unchecked(account_info.borrow_data_unchecked()) })
    }

    #[inline(always)]
    pub fn load_mut(account_info: &AccountInfo) -> Result<RefMut<'_, Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(RefMut::map(
            account_info.try_borrow_mut_data()?,
            |data| unsafe { Self::from_bytes_unchecked_mut(data) },
        ))
    }

    /// Load a mutable reference without borrow tracking.
    /// Use after CPI operations where the runtime may leave borrow state corrupted.
    #[inline(always)]
    pub unsafe fn load_mut_unchecked(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(unsafe { Self::from_bytes_unchecked_mut(
            account_info.borrow_mut_data_unchecked(),
        ) })
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const VestingAllocation) }
    }
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut VestingAllocation) }
    }

    #[inline(always)]
    pub fn recipient(&self) -> &Pubkey {
        &self.recipient
    }
    #[inline(always)]
    pub fn vesting_total(&self) -> u64 {
        self.vesting_total
    }
    #[inline(always)]
    pub fn withdrawn_amount(&self) -> u64 {
        self.withdrawn_amount
    }

    #[inline(always)]
    pub fn set_recipient(&mut self, recipient: Pubkey) -> Result<(), ProgramError> {
        self.recipient = recipient;

        Ok(())
    }
    #[inline(always)]
    pub fn set_vesting_total(&mut self, vesting_total: u64) -> Result<(), ProgramError> {
        self.vesting_total = vesting_total;

        Ok(())
    }
    #[inline(always)]
    pub fn set_withdrawn_amount(&mut self, withdrawn_amount: u64) -> Result<(), ProgramError> {
        self.withdrawn_amount = withdrawn_amount;

        Ok(())
    }

    pub fn set_inner(
        &mut self,
        recipient: Pubkey,
        vesting_total: u64,
        withdrawn_amount: u64,
    ) -> Result<(), ProgramError> {
        self.set_recipient(recipient)?;
        self.set_vesting_total(vesting_total)?;
        self.set_withdrawn_amount(withdrawn_amount)?;
        Ok(())
    }
}
