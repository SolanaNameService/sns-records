//! Validate and ETH signature

use crate::{
    error::SnsRecordsError,
    state::{
        record_header::RecordHeader,
        validation::{get_validation_length, Validation},
    },
    utils::{check_domain_owner, check_domain_parent},
};

use {
    crate::cpi,
    bonfida_utils::checks::check_account_owner,
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::keccak::Hasher,
    solana_program::secp256k1_recover::secp256k1_recover,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_program,
    },
    spl_name_service::state::NameRecordHeader,
    std::convert::TryInto,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The type of validation
    pub validation: Validation,
    /// The record enum as a string
    pub signature: Vec<u8>,
    /// The expected ETH public key
    pub expected_pubkey: Vec<u8>,
}

pub const ETH_PREFIX_BYTES: &[u8; 26] = b"\x19Ethereum Signed Message:\n";
pub const RECORD_SUFFIX: &[u8; 13] = b"\nFor record: ";
pub const STALENESS_SUFFIX: &[u8; 15] = b"\nStaleness ID: ";

///
/// The message to sign must contain the record public key & staleness ID
// +------------------------+------------------+------------------+------------------+------------------+------------------+------------------+
// | ETH_PREFIX_BYTES       | content_length   | content          | RECORD_SUFFIX    | record_key       | STALENESS_SUFFIX | staleness_id     |
// +------------------------+------------------+------------------+------------------+------------------+------------------+------------------+
// | "\x19Ethereum Signed   | Length of        | Actual content   | "\nFor record: " | Public key of    | "\nStaleness ID: " | Public key of   |
// | Message:\n"            | (content +       | to be signed     |                  | the record       |                  | the staleness    |
// |                        | record_key +     |                  |                  |                  |                  |                  |
// |                        | RECORD_SUFFIX +  |                  |                  |                  |                  |                  |
// |                        | staleness_id +   |                  |                  |                  |                  |                  |
// |                        | STALENESS_SUFFIX)|                  |                  |                  |                  |                  |
// +------------------------+------------------+------------------+------------------+------------------+------------------+------------------+
fn message_to_sign(content: &[u8], record_key: &Pubkey, staleness_id: &Pubkey) -> Vec<u8> {
    let mut buffer = Vec::new();
    let record_key_base58 = record_key.to_string();
    let staleness_id_base58 = staleness_id.to_string();
    let hex_encoded_content = hex::encode(content);

    let content_length = hex_encoded_content.len()
        + record_key_base58.len()
        + staleness_id_base58.len()
        + RECORD_SUFFIX.len()
        + STALENESS_SUFFIX.len();

    buffer.extend_from_slice(ETH_PREFIX_BYTES);
    buffer.extend_from_slice(content_length.to_string().as_bytes());
    buffer.extend_from_slice(hex_encoded_content.as_bytes());
    buffer.extend_from_slice(RECORD_SUFFIX);
    buffer.extend_from_slice(record_key_base58.as_bytes());
    buffer.extend_from_slice(STALENESS_SUFFIX);
    buffer.extend_from_slice(staleness_id_base58.as_bytes());

    buffer
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    /// The SPL token program account
    pub spl_name_service_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub fee_payer: &'a T,

    #[cons(writable)]
    /// The record account to create and post
    pub record: &'a T,

    #[cons(writable)]
    /// The domain name owning the record
    pub domain: &'a T,

    #[cons(writable, signer)]
    /// The domain owner
    pub domain_owner: &'a T,

    /// The SNS Record central state
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            record: next_account_info(accounts_iter)?,
            domain: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;

        // Check owners
        check_account_owner(accounts.record, &spl_name_service::ID)?;
        check_account_owner(accounts.domain, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;
        check_signer(accounts.domain_owner)?;

        Ok(accounts)
    }
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let Params {
        validation: _,
        signature,
        expected_pubkey,
    } = params;

    check_domain_owner(accounts.domain, accounts.domain_owner.key)?;
    check_domain_parent(accounts.record, accounts.domain.key)?;

    let (new_buffer, current_length, header) = {
        let record_data = accounts.record.data.borrow();
        let mut header = RecordHeader::from_buffer(&record_data);
        let (_, buffer) = record_data.split_at(NameRecordHeader::LEN + RecordHeader::LEN);

        let (staleness_id, rest) = buffer
            .split_at(get_validation_length(header.staleness_validation.try_into()?) as usize);
        let (_, content) = rest.split_at(get_validation_length(
            header.right_of_association_validation.try_into()?,
        ) as usize);

        // Minimally strip the XChain prefix
        // We're not worried about collisions between different chains as these are vanishingly unlikely
        // Unless the same private key is used across two different chains, in which case they're ownership-equivalent
        let staleness_id = &staleness_id[staleness_id.len().checked_sub(32).unwrap_or_default()..];

        // Implicitly means that if the staleness is not verified it's
        // impossible to verify the RoA
        let staleness_id_array: [u8; 32] = staleness_id
            .try_into()
            .map_err(|_| SnsRecordsError::OutOfBound)?;
        let buffer = message_to_sign(
            content,
            accounts.record.key,
            &Pubkey::from(staleness_id_array),
        );

        let recovery_id = signature[64]
            .checked_sub(27)
            .ok_or(SnsRecordsError::NumericalOverflow)?;

        let mut hasher = Hasher::default();
        hasher.hash(&buffer);
        let hash = hasher.result();

        let recovered_pubkey = secp256k1_recover(
            hash.as_ref(),
            recovery_id,
            signature.get(0..64).ok_or(SnsRecordsError::OutOfBound)?,
        )
        .map_err(|_| SnsRecordsError::Secp256k1Recover)?;

        // Hash the public key using Keccak-256
        let mut hasher = Hasher::default();
        hasher.hash(&recovered_pubkey.0);
        let output = hasher.result();

        // Take the last 20 bytes of the hash to get the Ethereum address
        let eth_address = output.0.get(12..).ok_or(SnsRecordsError::OutOfBound)?;

        if eth_address != expected_pubkey {
            return Err(SnsRecordsError::EthPubkeyMismatch.into());
        }

        /////////

        header.right_of_association_validation = Validation::Ethereum as u16;

        let mut new_buffer: Vec<u8> = vec![];
        new_buffer.extend_from_slice(staleness_id);
        new_buffer.extend_from_slice(&expected_pubkey);
        new_buffer.extend_from_slice(content);

        (new_buffer, buffer.len(), header)
    };

    if new_buffer.len() != current_length {
        cpi::resize_record(
            accounts.record,
            accounts.central_state,
            accounts.fee_payer,
            accounts.system_program,
            (new_buffer.len() + RecordHeader::LEN)
                .try_into()
                .map_err(|_| SnsRecordsError::NumericalOverflow)?,
        )?;
    }

    let header_bytes = bytemuck::bytes_of(&header);
    let data = [header_bytes, &new_buffer].concat();

    cpi::edit_record(&data, 0, accounts.record, accounts.central_state)?;

    Ok(())
}
