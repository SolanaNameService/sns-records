//! Write a RoA in the record

use std::convert::TryInto;

use spl_name_service::state::NameRecordHeader;

use crate::{
    state::{
        record_header::RecordHeader,
        validation::{get_validation_length, Validation},
    },
    utils::{check_domain_owner, check_domain_parent},
};

use {
    crate::cpi,
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub roa_id: Vec<u8>,
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
        check_account_key(
            accounts.system_program,
            &solana_system_interface::program::ID,
        )?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;

        // Check owners
        check_account_owner(accounts.record, &spl_name_service::ID)?;
        check_account_owner(accounts.domain, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.domain_owner)?;

        Ok(accounts)
    }
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let Params { roa_id } = params;

    check_domain_owner(accounts.domain, accounts.domain_owner.key)?;
    check_domain_parent(accounts.record, accounts.domain.key)?;

    let (new_buffer, current_length, header) = {
        let record_data = accounts.record.data.borrow();
        let mut header = RecordHeader::from_buffer(&record_data);
        let (_, buffer) = record_data.split_at(NameRecordHeader::LEN + RecordHeader::LEN);

        let mut new_buffer: Vec<u8> = vec![];

        let (staleness_id, rest) = buffer
            .split_at(get_validation_length(header.staleness_validation.try_into()?) as usize);
        let (_, content) = rest.split_at(get_validation_length(
            header.right_of_association_validation.try_into()?,
        ) as usize);

        header.right_of_association_validation = Validation::UnverifiedSolana as u16;
        new_buffer.extend_from_slice(staleness_id);
        new_buffer.extend_from_slice(&roa_id);
        new_buffer.extend_from_slice(content);

        (new_buffer, buffer.len(), header)
    };

    if new_buffer.len() != current_length {
        cpi::resize_record(
            accounts.record,
            accounts.central_state,
            accounts.fee_payer,
            accounts.system_program,
            (new_buffer.len() + RecordHeader::LEN) as u32,
        )?;
    }

    let header_bytes = bytemuck::bytes_of(&header);
    let data = [header_bytes, &new_buffer].concat();

    cpi::edit_record(&data, 0, accounts.record, accounts.central_state)?;

    Ok(())
}
