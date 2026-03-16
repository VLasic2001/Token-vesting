mod helpers;
use helpers::*;
use litesvm_token::CreateAssociatedTokenAccount;
use solana_sdk::{
    clock::Clock,
    signer::{keypair::Keypair, Signer},
};

#[test]
fn full_lifecycle() {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);
    let auth_pk = authority.pubkey();

    // 1. Create schedule: start=1000, cliff=200, step=100, total=600
    let (schedule_pda, schedule_ata) =
        create_test_schedule(&mut svm, &authority, &mint, 1000, 200, 100, 600, 99);

    // 2. Fund and allocate to 2 recipients
    let authority_ata = create_ata_and_mint(&mut svm, &authority, &mint, &auth_pk, 2_000_000);

    let recipient_a = Keypair::new();
    svm.airdrop(&recipient_a.pubkey(), 5_000_000_000).unwrap();
    let recipient_b = Keypair::new();
    svm.airdrop(&recipient_b.pubkey(), 5_000_000_000).unwrap();

    let (alloc_a, alloc_a_bump) = find_allocation_pda(&recipient_a.pubkey(), &schedule_pda);
    let (alloc_b, alloc_b_bump) = find_allocation_pda(&recipient_b.pubkey(), &schedule_pda);

    let ix_a = build_create_allocation_ix(
        &auth_pk, &authority_ata, &schedule_pda, &schedule_ata,
        &alloc_a, &recipient_a.pubkey(), 1_000_000, alloc_a_bump,
    );
    let ix_b = build_create_allocation_ix(
        &auth_pk, &authority_ata, &schedule_pda, &schedule_ata,
        &alloc_b, &recipient_b.pubkey(), 1_000_000, alloc_b_bump,
    );
    send_tx(&mut svm, &[ix_a, ix_b], &[&authority]);
    assert_eq!(get_token_balance(&svm, &schedule_ata), 2_000_000);

    // 3. Create recipient ATAs
    let ata_a = CreateAssociatedTokenAccount::new(&mut svm, &authority, &mint)
        .owner(&recipient_a.pubkey())
        .send()
        .unwrap();
    let ata_b = CreateAssociatedTokenAccount::new(&mut svm, &authority, &mint)
        .owner(&recipient_b.pubkey())
        .send()
        .unwrap();

    // 4. Fully vest, both withdraw
    svm.set_sysvar(&Clock { unix_timestamp: 5000, ..Default::default() });

    let wa = build_withdraw_ix(
        &recipient_a.pubkey(), &ata_a, &auth_pk,
        &schedule_pda, &schedule_ata, &alloc_a,
    );
    send_tx(&mut svm, &[wa], &[&recipient_a]);

    let wb = build_withdraw_ix(
        &recipient_b.pubkey(), &ata_b, &auth_pk,
        &schedule_pda, &schedule_ata, &alloc_b,
    );
    send_tx(&mut svm, &[wb], &[&recipient_b]);

    assert_eq!(get_token_balance(&svm, &ata_a), 1_000_000);
    assert_eq!(get_token_balance(&svm, &ata_b), 1_000_000);
    assert_eq!(get_token_balance(&svm, &schedule_ata), 0);

    // 5. Close schedule
    let close_ix = build_close_schedule_ix(&auth_pk, &schedule_pda, &schedule_ata, &mint);
    send_tx(&mut svm, &[close_ix], &[&authority]);

    let schedule_account = svm.get_account(&schedule_pda);
    assert!(schedule_account.is_none() || schedule_account.unwrap().data.is_empty());
}
