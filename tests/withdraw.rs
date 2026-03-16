mod helpers;
use helpers::*;
use solana_sdk::{
    clock::Clock,
    signer::{keypair::Keypair, Signer},
};
use litesvm_token::CreateAssociatedTokenAccount;

#[test]
fn before_cliff_fails() {
    let (mut svm, vs) = setup_vesting_scenario(1000, 500, 100, 1000, 1_000_000);
    svm.set_sysvar(&Clock { unix_timestamp: 1200, ..Default::default() });

    let ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&vs.recipient]);
}

#[test]
fn partial_after_some_steps() {
    // start=1000, cliff=200, step=100, total=1000
    // post-cliff=800, total_steps=8, per_step=125_000
    // at 1450: elapsed=250, steps=2, vested=250_000
    let (mut svm, vs) = setup_vesting_scenario(1000, 200, 100, 1000, 1_000_000);
    svm.set_sysvar(&Clock { unix_timestamp: 1450, ..Default::default() });

    let ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[ix], &[&vs.recipient]);

    assert_eq!(get_token_balance(&svm, &vs.recipient_ata), 250_000);
    assert_eq!(get_token_balance(&svm, &vs.schedule_ata), 750_000);
    assert!(svm.get_account(&vs.allocation_pda).is_some());
}

#[test]
fn full_vesting_closes_allocation() {
    let (mut svm, vs) = setup_vesting_scenario(1000, 200, 100, 1000, 1_000_000);
    svm.set_sysvar(&Clock { unix_timestamp: 5000, ..Default::default() });

    let ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[ix], &[&vs.recipient]);

    assert_eq!(get_token_balance(&svm, &vs.recipient_ata), 1_000_000);
    assert_eq!(get_token_balance(&svm, &vs.schedule_ata), 0);

    let alloc = svm.get_account(&vs.allocation_pda);
    assert!(alloc.is_none() || alloc.unwrap().data.is_empty());
}

#[test]
fn multiple_partial_withdrawals() {
    // start=1000, cliff=0, step=100, total=400
    // total_steps=4, per_step=250_000
    let (mut svm, vs) = setup_vesting_scenario(1000, 0, 100, 400, 1_000_000);

    // Step 1 at time=1100
    svm.set_sysvar(&Clock { unix_timestamp: 1100, ..Default::default() });
    let ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[ix], &[&vs.recipient]);
    assert_eq!(get_token_balance(&svm, &vs.recipient_ata), 250_000);

    // Step 3 at time=1300
    svm.expire_blockhash();
    svm.set_sysvar(&Clock { unix_timestamp: 1300, ..Default::default() });
    let ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[ix], &[&vs.recipient]);
    assert_eq!(get_token_balance(&svm, &vs.recipient_ata), 750_000);
}

#[test]
fn rounding_dust_fully_vested() {
    // 100 tokens / 3 steps = 33 per step → fully vested should still get 100
    let (mut svm, vs) = setup_vesting_scenario(1000, 0, 100, 300, 100);
    svm.set_sysvar(&Clock { unix_timestamp: 2000, ..Default::default() });

    let ix = build_withdraw_ix(
        &vs.recipient.pubkey(), &vs.recipient_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx(&mut svm, &[ix], &[&vs.recipient]);
    assert_eq!(get_token_balance(&svm, &vs.recipient_ata), 100);
}

#[test]
fn wrong_recipient_fails() {
    let (mut svm, vs) = setup_vesting_scenario(1000, 0, 100, 400, 1_000_000);
    svm.set_sysvar(&Clock { unix_timestamp: 1200, ..Default::default() });

    let fake = Keypair::new();
    svm.airdrop(&fake.pubkey(), 10_000_000_000).unwrap();
    let fake_ata = CreateAssociatedTokenAccount::new(&mut svm, &fake, &vs.mint)
        .send()
        .unwrap();

    let ix = build_withdraw_ix(
        &fake.pubkey(), &fake_ata, &vs.authority.pubkey(),
        &vs.schedule_pda, &vs.schedule_ata, &vs.allocation_pda,
    );
    send_tx_expect_err(&mut svm, &[ix], &[&fake]);
}
