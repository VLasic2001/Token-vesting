use {
    num_derive::FromPrimitive,
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror::Error,
};
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PinocchioError {
    // 0
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,

    #[error("Invalid Owner")]
    InvalidOwner,

    #[error("Invalid Address")]
    InvalidAddress,

    #[error("Invalid account data")]
    InvalidAccountData,

    #[error("Program Error")]
    ProgramError,
}

impl From<PinocchioError> for ProgramError {
    fn from(e: PinocchioError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl TryFrom<u32> for PinocchioError {
    type Error = ProgramError;
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(PinocchioError::NotRentExempt),
            1 => Ok(PinocchioError::InvalidOwner),
            2 => Ok(PinocchioError::InvalidAddress),
            3 => Ok(PinocchioError::InvalidAccountData),
            4 => Ok(PinocchioError::ProgramError),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

impl ToStr for PinocchioError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            PinocchioError::NotRentExempt => "Error: Lamport balance below rent-exempt threshold",
            PinocchioError::InvalidOwner => "Error: Invalid owner",
            PinocchioError::InvalidAddress => "Error: Invalid address",
            PinocchioError::InvalidAccountData => "Error Invalid account data",
            PinocchioError::ProgramError => "Program error",
        }
    }
}
