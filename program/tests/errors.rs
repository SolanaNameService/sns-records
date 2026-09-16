use sns_records::{entrypoint::process_instruction, utils::get_record_key_and_seeds};

use {
    solana_program::pubkey::Pubkey,
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        signer::{keypair::Keypair, Signer},
    },
    spl_name_service::state::NameRecordHeader,
    std::str::FromStr,
};

pub mod common;

#[tokio::test]
async fn test_functional() {
    use common::utils::sign_send_instructions;
    // Create program and test environment

    // Used for verifying the SOL RoA
    let guardian = Keypair::new();

    let bob = Keypair::new();

    // Dummy keypair hardcoded for verification signatures to remain constant between tests
    // Associated pubkey: 9K6vPLB1DqgznyA3CBKeZ3GnD8Fqo8vcvx2Vxkk5uwqN
    let alice = Keypair::new();
    let parent_name = Pubkey::from_str("4kG2PyqixXVUb2CEeNt1ZcVUEoomNssMe8C4hf4Dguch").unwrap();
    let domain = Pubkey::from_str("7nf2Rq9DxwQCTg1ZmEEB5VUVAzq6tGpsYxqJ6JHqyoTQ").unwrap();
    // Record key 6DsYWo7KBqQCB1RkSLy7DXtMFN1f6jXKQUsByJ1JiB5g

    let mut program_test = ProgramTest::new(
        "sns_records",
        sns_records::ID,
        processor!(process_instruction),
    );

    program_test.add_program("spl_name_service", spl_name_service::ID, None);

    program_test.add_account(
        alice.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        bob.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        guardian.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );

    ////
    // Set up domain name
    ////
    let domain_record_header = NameRecordHeader {
        parent_name,
        owner: alice.pubkey(),
        class: Pubkey::default(),
    };
    program_test.add_account(
        domain,
        Account {
            lamports: 100_000_000_000,
            data: borsh::to_vec(&domain_record_header).unwrap(),
            owner: spl_name_service::ID,
            ..Account::default()
        },
    );

    ////
    // Set up a domain for Bob
    ////
    let bob_domain_key = Keypair::new().pubkey();
    let domain_record_header = NameRecordHeader {
        parent_name,
        owner: bob.pubkey(),
        class: Pubkey::default(),
    };
    program_test.add_account(
        bob_domain_key,
        Account {
            lamports: 100_000_000_000,
            data: borsh::to_vec(&domain_record_header).unwrap(),
            owner: spl_name_service::ID,
            ..Account::default()
        },
    );

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    let content_length = 10;
    let record = "SOL";

    let (record_key, _) = get_record_key_and_seeds(&domain, record);

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Only domain owner can allocate record
    ////
    let ix = sns_records::instruction::allocate_record(
        sns_records::instruction::allocate_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::allocate_record::Params {
            content_length,
            record: record.to_owned(),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Only domain owner can allocate and post record
    ////
    let content = "some random content".as_bytes();
    let ix = sns_records::instruction::allocate_and_post_record(
        sns_records::instruction::allocate_and_post_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::allocate_and_post_record::Params {
            content: content.to_vec(),
            record: record.to_owned(),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Create a record owned by Alice
    ////
    let ix = sns_records::instruction::allocate_and_post_record(
        sns_records::instruction::allocate_and_post_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::allocate_and_post_record::Params {
            content: content.to_vec(),
            record: record.to_owned(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to edit Alice's record
    ////
    let ix = sns_records::instruction::edit_record(
        sns_records::instruction::edit_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::edit_record::Params {
            content: vec![],
            record: record.to_owned(),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to edit Alice's record by passing a domain he owns
    ////
    let ix = sns_records::instruction::edit_record(
        sns_records::instruction::edit_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &bob_domain_key,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::edit_record::Params {
            content: vec![],
            record: record.to_owned(),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to delete Alice's record by passing a domain he owns
    ////
    let ix = sns_records::instruction::delete_record(
        sns_records::instruction::delete_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &bob_domain_key,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::delete_record::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to verify staleness Alice's record by passing a domain he owns
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &bob_domain_key,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &bob.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: true },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to validate staleness record for Alice
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &bob.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: true },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Alice tries to validate non-existant RoA record for Alice
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &alice.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: false },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to validate non-existant RoA record for Alice
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &bob.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: false },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to write RoA record for Alice
    ////
    let ix = sns_records::instruction::write_roa(
        sns_records::instruction::write_roa::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::write_roa::Params {
            roa_id: bob.pubkey().to_bytes().to_vec(),
        },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Alice writes RoA
    ////
    let ix = sns_records::instruction::write_roa(
        sns_records::instruction::write_roa::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::write_roa::Params {
            roa_id: guardian.pubkey().to_bytes().to_vec(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to validate wrong RoA record for Alice
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &bob.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: false },
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to delete record for Alice
    ////
    let ix = sns_records::instruction::delete_record(
        sns_records::instruction::delete_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &bob.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::delete_record::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Guardian verifies RoA for Alice
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &guardian.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &guardian.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: false },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&guardian])
        .await
        .unwrap();
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Bob tries to unverify RoA for Alice
    ////
    let ix = sns_records::instruction::unverify_roa(
        sns_records::instruction::unverify_roa::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            central_state: &sns_records::central_state::KEY,
            verifier: &bob.pubkey(),
        },
        sns_records::instruction::unverify_roa::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Alice tries to unverify RoA for Alice
    ////
    let ix = sns_records::instruction::unverify_roa(
        sns_records::instruction::unverify_roa::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            central_state: &sns_records::central_state::KEY,
            verifier: &alice.pubkey(),
        },
        sns_records::instruction::unverify_roa::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(res.is_err());
    ////////////////////////////////////////////////////////////////////////////////////////////////////
}
