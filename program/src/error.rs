use {num_derive::FromPrimitive, solana_program::program_error::ProgramError, thiserror::Error};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum SnsRecordsError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
    #[error("Unsupported validation")]
    UnsupportedValidation,
    #[error("Could not recover public key")]
    Secp256k1Recover,
    #[error("ETH public key mismatch")]
    EthPubkeyMismatch,
    #[error("Wrong domain owner")]
    WrongDomainOwner,
    #[error("Numerical overflow")]
    NumericalOverflow,
    #[error("Array out of bound")]
    OutOfBound,
    #[error("Invalid verifier")]
    InvalidVerifier,
    #[error("Wrong parent")]
    WrongParent,
    #[error("Wrong class")]
    WrongClass,
}

impl From<SnsRecordsError> for ProgramError {
    fn from(e: SnsRecordsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
