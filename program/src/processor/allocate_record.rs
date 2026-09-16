//! Allocate record account

use bonfida_utils::checks::check_account_owner;

use crate::{
    state::record_header::RecordHeader,
    utils::{check_domain_owner, get_record_key_and_seeds},
};

use {
    crate::cpi,
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    // sns_sdk::record::Record,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The record length
    pub content_length: u32,

    pub record: String,
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
    /// The record account to create
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
        check_account_owner(accounts.record, &solana_system_interface::program::ID)?;
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

    let hashed = crate::utils::get_hashed_name(params.record.as_str());
    let (key, _) = get_record_key_and_seeds(accounts.domain.key, &params.record);
    check_account_key(accounts.record, &key)?;

    cpi::allocate_record(
        params.content_length + RecordHeader::LEN as u32,
        &hashed,
        accounts.record,
        accounts.fee_payer,
        accounts.domain,
        accounts.domain_owner,
        accounts.central_state,
        accounts.system_program,
    )?;

    Ok(())
}
