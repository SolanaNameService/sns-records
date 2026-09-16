//! Edit the record content

use bonfida_utils::checks::check_account_owner;
use solana_program::program_pack::Pack;
use spl_name_service::state::NameRecordHeader;

use crate::{
    state::record_header::RecordHeader,
    utils::{check_domain_owner, check_domain_parent},
};

use {
    crate::cpi,
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub record: String,
    pub content: Vec<u8>,
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
    /// The record account to edit
    pub record: &'a T,

    #[cons(writable)]
    pub domain: &'a T,

    #[cons(writable, signer)]
    pub domain_owner: &'a T,

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
        check_signer(accounts.fee_payer)?;
        check_signer(accounts.domain_owner)?;

        Ok(accounts)
    }
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;

    check_domain_owner(accounts.domain, accounts.domain_owner.key)?;
    check_domain_parent(accounts.record, accounts.domain.key)?;

    let header = RecordHeader::new(params.content.len() as u32);

    let header_bytes = bytemuck::bytes_of(&header);
    let data = [header_bytes, &params.content].concat();

    if accounts.record.data_len() - NameRecordHeader::LEN != data.len() {
        cpi::resize_record(
            accounts.record,
            accounts.central_state,
            accounts.fee_payer,
            accounts.system_program,
            data.len() as u32,
        )?;
    }

    cpi::edit_record(&data, 0, accounts.record, accounts.central_state)?;

    Ok(())
}
