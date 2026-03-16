use crate::errors::PinocchioError;
use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_system::instructions::CreateAccount;
use pinocchio::sysvars::{rent::Rent, Sysvar};

pub struct SignerAccount;
impl SignerAccount {
    pub fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(PinocchioError::InvalidOwner.into());
        }

        Ok(())
    }
}

pub struct MintInterface;
impl MintInterface {
    pub fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_token::ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }

        Ok(())
    }
}

pub struct TokenAccount;
impl TokenAccount {
    pub fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_token::ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }

        Ok(())
    }
}

pub struct AssociatedTokenAccount;
impl AssociatedTokenAccount {
    pub fn check(
        account: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        TokenAccount::check(account)?;

        if find_program_address(
            &[authority.key(), token_program.key(), mint.key()],
            &pinocchio_associated_token_account::ID,
        )
        .0
        .ne(account.key())
        {
            return Err(PinocchioError::InvalidAddress.into());
        }

        Ok(())
    }

    pub fn init(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        Create {
            funding_account: payer,
            account,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }
        .invoke()
    }

    pub fn init_if_needed(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        match Self::check(account, payer, mint, token_program) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, owner, system_program, token_program),
        }
    }
}

pub struct ProgramAccount;
impl ProgramAccount {
    pub fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&crate::ID) {
            return Err(PinocchioError::InvalidAccountData.into());
        }

        // if !account.data_len().ne(&crate::state::)

        Ok(())
    }

    pub fn init<'a, T: Sized>(
        payer: &AccountInfo,
        account: &AccountInfo,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> ProgramResult {
        let lamports = Rent::get()
            .map_err(|_| ProgramError::InvalidAccountData)?
            .minimum_balance(space);

        let signer = [Signer::from(seeds)];

        CreateAccount {
            from: payer,
            to: account,
            lamports,
            owner: &crate::ID,
            space: space as u64,
        }
        .invoke_signed(&signer)?;

        Ok(())
    }

    pub fn close(account: &AccountInfo, destination: &AccountInfo) -> ProgramResult {
        {
            let mut data = account.try_borrow_mut_data()?;
            data[0] = 0xff;
        }
        *destination.try_borrow_mut_lamports()? += *account.try_borrow_lamports()?;
        account.resize(1)?;
        account.close()
    }
}
