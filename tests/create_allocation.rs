mod helpers;
use helpers::*;
use solana_sdk::signer::{keypair::Keypair, Signer};

#[test]
fn success() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let (schedule_pda, schedule_ata) =
        create_test_schedule(&mut svm, &authority, &mint, 1_000_000, 100, 50, 500, 1);

    let authority_ata = create_ata_and_mint(&mut svm, &authority, &mint, &auth_pk, 1_000_000);

    let recipient = Keypair::new();
    let (allocation_pda, allocation_bump) = find_allocation_pda(&recipient.pubkey(), &schedule_pda);

    let ix = build_create_allocation_ix(
        &auth_pk, &authority_ata, &schedule_pda, &schedule_ata,
        &allocation_pda, &recipient.pubkey(), 500_000, allocation_bump,
    );
    send_tx(&mut svm, &[ix], &[&authority]);

    let alloc_account = svm.get_account(&allocation_pda).unwrap();
    assert_eq!(alloc_account.owner, program_id());
    assert_eq!(get_token_balance(&svm, &schedule_ata), 500_000);
}

#[test]
fn zero_amount() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let (schedule_pda, schedule_ata) =
        create_test_schedule(&mut svm, &authority, &mint, 1_000_000, 100, 50, 500, 1);
    let authority_ata = create_ata_and_mint(&mut svm, &authority, &mint, &auth_pk, 1_000_000);

    let recipient = Keypair::new();
    let (allocation_pda, allocation_bump) = find_allocation_pda(&recipient.pubkey(), &schedule_pda);

    let ix = build_create_allocation_ix(
        &auth_pk, &authority_ata, &schedule_pda, &schedule_ata,
        &allocation_pda, &recipient.pubkey(), 0, allocation_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&authority]);
}

#[test]
fn wrong_authority() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);

    let (schedule_pda, schedule_ata) =
        create_test_schedule(&mut svm, &authority, &mint, 1_000_000, 100, 50, 500, 1);

    let fake = Keypair::new();
    svm.airdrop(&fake.pubkey(), 10_000_000_000).unwrap();
    let fake_ata = create_ata_and_mint(&mut svm, &authority, &mint, &fake.pubkey(), 1_000_000);

    let recipient = Keypair::new();
    let (allocation_pda, allocation_bump) = find_allocation_pda(&recipient.pubkey(), &schedule_pda);

    let ix = build_create_allocation_ix(
        &fake.pubkey(), &fake_ata, &schedule_pda, &schedule_ata,
        &allocation_pda, &recipient.pubkey(), 500_000, allocation_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&fake]);
}
