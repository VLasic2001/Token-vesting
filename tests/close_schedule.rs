mod helpers;
use helpers::*;
use solana_sdk::{
    clock::Clock,
    signer::{keypair::Keypair, Signer},
};

#[test]
fn success() {
    let (mut svm, vs) = setup_vesting_scenario(1000, 0, 100, 400, 1_000_000);

    // Fully vest and withdraw
    svm.set_sysvar(&Clock { unix_timestamp: 5000, ..Default::default() });
    let withdraw_ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[withdraw_ix], &[&vs.recipient]);

    // Close schedule
    let close_ix = build_close_schedule_ix(
        &vs.authority.pubkey(), &vs.schedule_pda, &vs.schedule_ata, &vs.mint,
    );
    send_tx(&mut svm, &[close_ix], &[&vs.authority]);

    let schedule_account = svm.get_account(&vs.schedule_pda);
    assert!(schedule_account.is_none() || schedule_account.unwrap().data.is_empty());
}

#[test]
fn with_balance_fails() {
    let (mut svm, vs) = setup_vesting_scenario(1000, 0, 100, 400, 1_000_000);

    // Don't withdraw, try to close
    let close_ix = build_close_schedule_ix(
        &vs.authority.pubkey(), &vs.schedule_pda, &vs.schedule_ata, &vs.mint,
    );
    send_tx_expect_err(&mut svm, &[close_ix], &[&vs.authority]);
}

#[test]
fn wrong_authority_fails() {
    let (mut svm, vs) = setup_vesting_scenario(1000, 0, 100, 400, 1_000_000);

    // Fully vest and withdraw
    svm.set_sysvar(&Clock { unix_timestamp: 5000, ..Default::default() });
    let withdraw_ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[withdraw_ix], &[&vs.recipient]);

    // Try to close with wrong authority
    let fake = Keypair::new();
    svm.airdrop(&fake.pubkey(), 10_000_000_000).unwrap();
    let close_ix = build_close_schedule_ix(
        &fake.pubkey(), &vs.schedule_pda, &vs.schedule_ata, &vs.mint,
    );
    send_tx_expect_err(&mut svm, &[close_ix], &[&fake]);
}
