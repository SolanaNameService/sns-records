pub use crate::processor::{
    allocate_and_post_record, allocate_record, delete_record, edit_record, unverify_roa,
    validate_ethereum_signature, validate_solana_signature, write_roa,
};
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Allocate record account
    ///
    /// | Index | Writable | Signer | Description                       |
    /// | ------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account        |
    /// | 1     | ❌        | ❌      | The SPL token program account     |
    /// | 2     | ✅        | ✅      | The fee payer account             |
    /// | 3     | ✅        | ❌      | The record account to create      |
    /// | 4     | ✅        | ❌      | The domain name owning the record |
    /// | 5     | ✅        | ✅      | The domain owner                  |
    /// | 6     | ❌        | ❌      | The SNS Record central state      |
    AllocateRecord,
    /// Allocate record account
    ///
    /// | Index | Writable | Signer | Description                           |
    /// | ----------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account            |
    /// | 1     | ❌        | ❌      | The SPL token program account         |
    /// | 2     | ✅        | ✅      | The fee payer account                 |
    /// | 3     | ✅        | ❌      | The record account to create and post |
    /// | 4     | ✅        | ❌      | The domain name owning the record     |
    /// | 5     | ✅        | ✅      | The domain owner                      |
    /// | 6     | ❌        | ❌      | The SNS Record central state          |
    AllocateAndPostRecord,
    /// Edit the record content
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account    |
    /// | 1     | ❌        | ❌      | The SPL token program account |
    /// | 2     | ✅        | ✅      | The fee payer account         |
    /// | 3     | ✅        | ❌      | The record account to edit    |
    /// | 4     | ✅        | ❌      |                               |
    /// | 5     | ✅        | ✅      |                               |
    /// | 6     | ❌        | ❌      |                               |
    EditRecord,
    /// Validate a RoA or Staleness via Solana signature
    ///
    /// | Index | Writable | Signer | Description                           |
    /// | ----------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account            |
    /// | 1     | ❌        | ❌      | The SPL token program account         |
    /// | 2     | ✅        | ✅      | The fee payer account                 |
    /// | 3     | ✅        | ❌      | The record account to create and post |
    /// | 4     | ✅        | ❌      | The domain name owning the record     |
    /// | 5     | ✅        | ❌      | The domain owner                      |
    /// | 6     | ❌        | ❌      | The SNS Record central state          |
    /// | 7     | ✅        | ✅      | The RoA/Staleness verifier public key |
    ValidateSolanaSignature,
    /// Validate and ETH signature
    ///
    /// | Index | Writable | Signer | Description                           |
    /// | ----------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account            |
    /// | 1     | ❌        | ❌      | The SPL token program account         |
    /// | 2     | ✅        | ✅      | The fee payer account                 |
    /// | 3     | ✅        | ❌      | The record account to create and post |
    /// | 4     | ✅        | ❌      | The domain name owning the record     |
    /// | 5     | ✅        | ✅      | The domain owner                      |
    /// | 6     | ❌        | ❌      | The SNS Record central state          |
    ValidateEthereumSignature,
    /// Delete a record account
    ///
    /// | Index | Writable | Signer | Description                       |
    /// | ------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account        |
    /// | 1     | ❌        | ❌      | The SPL token program account     |
    /// | 2     | ✅        | ✅      | The fee payer account             |
    /// | 3     | ✅        | ❌      | The record account to delete      |
    /// | 4     | ✅        | ❌      | The domain name owning the record |
    /// | 5     | ✅        | ✅      | The domain owner                  |
    /// | 6     | ❌        | ❌      | The SNS Record central state      |
    DeleteRecord,
    /// Write a RoA in the record
    ///
    /// | Index | Writable | Signer | Description                           |
    /// | ----------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account            |
    /// | 1     | ❌        | ❌      | The SPL token program account         |
    /// | 2     | ✅        | ✅      | The fee payer account                 |
    /// | 3     | ✅        | ❌      | The record account to create and post |
    /// | 4     | ✅        | ❌      | The domain name owning the record     |
    /// | 5     | ✅        | ✅      | The domain owner                      |
    /// | 6     | ❌        | ❌      | The SNS Record central state          |
    WriteRoa,
    /// Unverify a RoA in the record
    ///
    /// | Index | Writable | Signer | Description                           |
    /// | ----------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account            |
    /// | 1     | ❌        | ❌      | The SPL token program account         |
    /// | 2     | ✅        | ✅      | The fee payer account                 |
    /// | 3     | ✅        | ❌      | The record account to create and post |
    /// | 4     | ✅        | ❌      | The domain name owning the record     |
    /// | 5     | ❌        | ❌      | The SNS Record central state          |
    /// | 6     | ✅        | ✅      | The current ROA verifier              |
    UnverifyRoa,
}
#[allow(missing_docs)]
pub fn allocate_record(
    accounts: allocate_record::Accounts<Pubkey>,
    params: allocate_record::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::AllocateRecord as u8, params)
}
#[allow(missing_docs)]
pub fn allocate_and_post_record(
    accounts: allocate_and_post_record::Accounts<Pubkey>,
    params: allocate_and_post_record::Params,
) -> Instruction {
    accounts.get_instruction(
        crate::ID,
        ProgramInstruction::AllocateAndPostRecord as u8,
        params,
    )
}
pub fn edit_record(
    accounts: edit_record::Accounts<Pubkey>,
    params: edit_record::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::EditRecord as u8, params)
}
pub fn validate_ethereum_signature(
    accounts: validate_ethereum_signature::Accounts<Pubkey>,
    params: validate_ethereum_signature::Params,
) -> Instruction {
    accounts.get_instruction(
        crate::ID,
        ProgramInstruction::ValidateEthereumSignature as u8,
        params,
    )
}
pub fn validate_solana_signature(
    accounts: validate_solana_signature::Accounts<Pubkey>,
    params: validate_solana_signature::Params,
) -> Instruction {
    accounts.get_instruction(
        crate::ID,
        ProgramInstruction::ValidateSolanaSignature as u8,
        params,
    )
}
pub fn delete_record(
    accounts: delete_record::Accounts<Pubkey>,
    params: delete_record::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::DeleteRecord as u8, params)
}
pub fn write_roa(accounts: write_roa::Accounts<Pubkey>, params: write_roa::Params) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::WriteRoa as u8, params)
}
pub fn unverify_roa(
    accounts: unverify_roa::Accounts<Pubkey>,
    params: unverify_roa::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::UnverifyRoa as u8, params)
}
