use std::str::FromStr;

use sns_records::{
    entrypoint::process_instruction,
    instruction::validate_solana_signature,
    state::{record_header::RecordHeader, validation::Validation},
    utils::get_record_key_and_seeds,
};
use solana_program::program_pack::Pack;

use {
    solana_program::pubkey::Pubkey,
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        signer::{keypair::Keypair, Signer},
    },
    spl_name_service::state::NameRecordHeader,
};

pub mod common;

#[tokio::test]
async fn test_functional() {
    use common::utils::sign_send_instructions;
    // Create program and test environment

    // Used for verifying the SOL RoA
    let bob = Keypair::new();

    // Dummy keypair hardcoded for verification signatures to remain constant between tests
    // Associated pubkey: 9K6vPLB1DqgznyA3CBKeZ3GnD8Fqo8vcvx2Vxkk5uwqN
    let alice = Keypair::new_from_array([
        42, 185, 156, 155, 46, 95, 163, 247, 19, 215, 251, 222, 166, 74, 236, 11, 8, 248, 245, 184,
        40, 127, 236, 213, 229, 186, 144, 210, 89, 137, 115, 230,
    ]);
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
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    ////
    // Allocate a record
    ////

    let content_length = 10;
    let record = "SOL";

    let (record_key, _) = get_record_key_and_seeds(&domain, record);

    let ix = sns_records::instruction::allocate_record(
        sns_records::instruction::allocate_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::allocate_record::Params {
            content_length,
            record: record.to_owned(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, 0);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::None as u16);
    assert_eq!(
        account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [0; 10]
    );

    ////
    // Delete a record
    ////
    let ix = sns_records::instruction::delete_record(
        sns_records::instruction::delete_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::delete_record::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap();
    assert!(account.is_none());

    ////
    // Allocate and post a record
    ////
    let content = "some random content".as_bytes();
    let record = "CNAME";

    let (record_key, _) = get_record_key_and_seeds(&domain, record);

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

    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::None as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        content
    );

    ////
    // Edit a record (increase size)
    ////

    let content = "some different random content".as_bytes();

    let ix = sns_records::instruction::edit_record(
        sns_records::instruction::edit_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::edit_record::Params {
            content: content.to_vec(),
            record: record.to_owned(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::None as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        content
    );

    ////
    // Edit a record (decrease size)
    ////

    let content = "a".as_bytes();

    let ix = sns_records::instruction::edit_record(
        sns_records::instruction::edit_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::edit_record::Params {
            content: content.to_vec(),
            record: record.to_owned(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::None as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        content
    );

    ////
    // Verify SOL
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
        sns_records::instruction::validate_solana_signature::Params { staleness: true },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [alice.pubkey().as_ref(), content].concat()
    );

    ////
    // Write RoA
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
            roa_id: bob.pubkey().to_bytes().to_vec(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::UnverifiedSolana as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [alice.pubkey().as_ref(), bob.pubkey().as_ref(), content].concat()
    );

    ////
    // Verify RoA Solana
    ////
    let ix = sns_records::instruction::validate_solana_signature(
        sns_records::instruction::validate_solana_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &bob.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
            verifier: &bob.pubkey(),
        },
        sns_records::instruction::validate_solana_signature::Params { staleness: false },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::Solana as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [alice.pubkey().as_ref(), bob.pubkey().as_ref(), content].concat()
    );

    ////
    // Unverify RoA
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
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [alice.pubkey().as_ref(), content].concat()
    );

    ////
    // Verify ETH
    ////
    let content = hex::decode("4bfbfd1e018f9f27eeb788160579daf7e2cd7da7").unwrap();

    let ix = sns_records::instruction::edit_record(
        sns_records::instruction::edit_record::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::edit_record::Params {
            content: content.clone(),
            record: record.to_owned(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::None as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        content
    );

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
        sns_records::instruction::validate_solana_signature::Params { staleness: true },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [alice.pubkey().as_ref(), &content].concat()
    );

    let expected_pubkey = vec![
        75, 251, 253, 30, 1, 143, 159, 39, 238, 183, 136, 22, 5, 121, 218, 247, 226, 205, 125, 167,
    ];
    let ix = sns_records::instruction::validate_ethereum_signature(
        sns_records::instruction::validate_ethereum_signature::Accounts {
            system_program: &solana_system_interface::program::ID,
            spl_name_service_program: &spl_name_service::ID,
            fee_payer: &alice.pubkey(),
            record: &record_key,
            domain: &domain,
            domain_owner: &alice.pubkey(),
            central_state: &sns_records::central_state::KEY,
        },
        sns_records::instruction::validate_ethereum_signature::Params {
            validation: sns_records::state::validation::Validation::Ethereum,
            signature: vec![
                4, 40, 252, 146, 134, 208, 96, 87, 138, 248, 93, 73, 18, 149, 165, 176, 211, 225,
                15, 75, 19, 90, 251, 192, 49, 183, 6, 196, 33, 75, 48, 139, 95, 224, 244, 176, 178,
                249, 110, 250, 25, 5, 229, 185, 86, 115, 119, 184, 22, 74, 199, 214, 93, 145, 73,
                214, 169, 91, 76, 172, 185, 236, 35, 194, 28,
            ],
            expected_pubkey: expected_pubkey.clone(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::Ethereum as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [alice.pubkey().as_ref(), &expected_pubkey, &content].concat()
    );

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////
    // Alice transfers to Bob, Bob signs the staleness -> roa must be `Validation::None`
    ////

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![
            spl_name_service::instruction::transfer(
                spl_name_service::ID,
                bob.pubkey(),
                domain,
                alice.pubkey(),
                None,
            )
            .unwrap(),
            validate_solana_signature(
                validate_solana_signature::Accounts {
                    system_program: &solana_system_interface::program::ID,
                    spl_name_service_program: &spl_name_service::ID,
                    fee_payer: &alice.pubkey(),
                    record: &record_key,
                    domain: &domain,
                    domain_owner: &bob.pubkey(),
                    central_state: &sns_records::central_state::KEY,
                    verifier: &bob.pubkey(),
                },
                validate_solana_signature::Params { staleness: true },
            ),
        ],
        vec![&alice, &bob],
    )
    .await
    .unwrap();

    ////
    // State verification
    ////
    let account = prg_test_ctx
        .banks_client
        .get_account(record_key)
        .await
        .unwrap()
        .unwrap();
    let record_hd = RecordHeader::from_buffer(&account.data);
    assert_eq!(record_hd.content_length, content.len() as u32);
    assert_eq!(
        record_hd.right_of_association_validation,
        Validation::None as u16
    );
    assert_eq!(record_hd.staleness_validation, Validation::Solana as u16);
    assert_eq!(
        &account.data[NameRecordHeader::LEN + RecordHeader::LEN..],
        [bob.pubkey().as_ref(), &expected_pubkey, &content].concat()
    );
}
