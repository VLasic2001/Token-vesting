mod helpers;
use helpers::*;
use solana_sdk::signer::{keypair::Keypair, Signer};

#[test]
fn success() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);

    let (schedule_pda, schedule_ata) =
        create_test_schedule(&mut svm, &authority, &mint, 1_000_000, 100, 50, 500, 1);

    let schedule_account = svm.get_account(&schedule_pda).unwrap();
    assert_eq!(schedule_account.owner, program_id());

    let ata_account = svm.get_account(&schedule_ata).unwrap();
    assert_eq!(ata_account.owner, token_program_id());
}

#[test]
fn step_duration_zero() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let seed: u64 = 1;
    let (schedule_pda, schedule_bump) = find_schedule_pda(seed, &mint);
    let schedule_ata = find_ata(&schedule_pda, &mint);

    let ix = build_create_schedule_ix(
        &auth_pk, &mint, &schedule_pda, &schedule_ata,
        1_000_000, 100, 0, 500, seed, schedule_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&authority]);
}

#[test]
fn total_vesting_time_zero() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let seed: u64 = 1;
    let (schedule_pda, schedule_bump) = find_schedule_pda(seed, &mint);
    let schedule_ata = find_ata(&schedule_pda, &mint);

    let ix = build_create_schedule_ix(
        &auth_pk, &mint, &schedule_pda, &schedule_ata,
        1_000_000, 0, 50, 0, seed, schedule_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&authority]);
}

#[test]
fn cliff_greater_than_total() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let seed: u64 = 1;
    let (schedule_pda, schedule_bump) = find_schedule_pda(seed, &mint);
    let schedule_ata = find_ata(&schedule_pda, &mint);

    let ix = build_create_schedule_ix(
        &auth_pk, &mint, &schedule_pda, &schedule_ata,
        1_000_000, 600, 50, 500, seed, schedule_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&authority]);
}

#[test]
fn negative_cliff() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let seed: u64 = 1;
    let (schedule_pda, schedule_bump) = find_schedule_pda(seed, &mint);
    let schedule_ata = find_ata(&schedule_pda, &mint);

    let ix = build_create_schedule_ix(
        &auth_pk, &mint, &schedule_pda, &schedule_ata,
        1_000_000, -1, 50, 500, seed, schedule_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&authority]);
}

#[test]
fn step_larger_than_post_cliff() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    let seed: u64 = 1;
    let (schedule_pda, schedule_bump) = find_schedule_pda(seed, &mint);
    let schedule_ata = find_ata(&schedule_pda, &mint);

    // total - cliff = 500 - 450 = 50, step = 100 > 50
    let ix = build_create_schedule_ix(
        &auth_pk, &mint, &schedule_pda, &schedule_ata,
        1_000_000, 450, 100, 500, seed, schedule_bump,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&authority]);
}
