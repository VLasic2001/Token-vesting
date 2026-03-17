use agave_feature_set::FeatureSet;
use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    transaction::Transaction,
};

pub const PROGRAM_ID_BYTES: [u8; 32] = [
    0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07, 0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
    0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07, 0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7,
];

pub fn program_id() -> Pubkey {
    Pubkey::from(PROGRAM_ID_BYTES)
}

pub fn token_program_id() -> Pubkey {
    spl_token::ID
}

pub fn system_program_id() -> Pubkey {
    Pubkey::from_str_const("11111111111111111111111111111111")
}

pub fn ata_program_id() -> Pubkey {
    spl_associated_token_account::ID
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut feature_set = FeatureSet::all_enabled();
    feature_set.deactivate(&agave_feature_set::account_data_direct_mapping::id());
    let mut svm = LiteSVM::new().with_feature_set(feature_set).with_builtins();
    svm.add_program_from_file(program_id(), "target/deploy/vesting.so")
        .unwrap();
    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();
    (svm, authority)
}

// =============================================================================
// PDA helpers
// =============================================================================

pub fn find_schedule_pda(seed: u64, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"vesting", &seed.to_le_bytes(), mint.as_ref()],
        &program_id(),
    )
}

pub fn find_allocation_pda(recipient: &Pubkey, schedule: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"allocation", recipient.as_ref(), schedule.as_ref()],
        &program_id(),
    )
}

pub fn find_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(owner, mint)
}

// =============================================================================
// Instruction builders
// =============================================================================

pub fn build_create_schedule_ix(
    creator: &Pubkey,
    mint: &Pubkey,
    schedule: &Pubkey,
    schedule_ata: &Pubkey,
    start_time: i64,
    cliff_time: i64,
    step_duration: i64,
    total_vesting_time: i64,
    seed: u64,
    schedule_bump: u8,
) -> Instruction {
    let mut data = vec![1u8];
    data.extend_from_slice(&start_time.to_le_bytes());
    data.extend_from_slice(&cliff_time.to_le_bytes());
    data.extend_from_slice(&step_duration.to_le_bytes());
    data.extend_from_slice(&total_vesting_time.to_le_bytes());
    data.extend_from_slice(&seed.to_le_bytes());
    data.push(schedule_bump);

    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(*creator, true),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(*schedule, false),
            AccountMeta::new(*schedule_ata, false),
            AccountMeta::new_readonly(system_program_id(), false),
            AccountMeta::new_readonly(token_program_id(), false),
            AccountMeta::new_readonly(ata_program_id(), false),
        ],
        data,
    }
}

pub fn build_create_allocation_ix(
    creator: &Pubkey,
    creator_ata: &Pubkey,
    schedule: &Pubkey,
    schedule_ata: &Pubkey,
    allocation: &Pubkey,
    recipient: &Pubkey,
    amount: u64,
    allocation_bump: u8,
) -> Instruction {
    let mut data = vec![2u8];
    data.extend_from_slice(recipient.as_ref());
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(allocation_bump);

    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(*creator, true),
            AccountMeta::new(*creator_ata, false),
            AccountMeta::new_readonly(*schedule, false),
            AccountMeta::new(*schedule_ata, false),
            AccountMeta::new(*allocation, false),
            AccountMeta::new_readonly(system_program_id(), false),
            AccountMeta::new_readonly(token_program_id(), false),
        ],
        data,
    }
}

pub fn build_withdraw_ix(
    recipient: &Pubkey,
    recipient_ata: &Pubkey,
    authority: &Pubkey,
    schedule: &Pubkey,
    schedule_ata: &Pubkey,
    allocation: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(*recipient, true),
            AccountMeta::new(*recipient_ata, false),
            AccountMeta::new(*authority, false),
            AccountMeta::new_readonly(*schedule, false),
            AccountMeta::new(*schedule_ata, false),
            AccountMeta::new(*allocation, false),
            AccountMeta::new_readonly(token_program_id(), false),
        ],
        data: vec![3u8],
    }
}

pub fn build_close_schedule_ix(
    authority: &Pubkey,
    schedule: &Pubkey,
    schedule_ata: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(*schedule, false),
            AccountMeta::new(*schedule_ata, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(token_program_id(), false),
        ],
        data: vec![4u8],
    }
}

// =============================================================================
// Transaction helpers
// =============================================================================

pub fn send_tx(svm: &mut LiteSVM, ixs: &[Instruction], signers: &[&Keypair]) {
    let tx = Transaction::new_signed_with_payer(
        ixs,
        Some(&signers[0].pubkey()),
        signers,
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();
}

pub fn send_tx_expect_err(svm: &mut LiteSVM, ixs: &[Instruction], signers: &[&Keypair]) {
    let tx = Transaction::new_signed_with_payer(
        ixs,
        Some(&signers[0].pubkey()),
        signers,
        svm.latest_blockhash(),
    );
    assert!(svm.send_transaction(tx).is_err());
}

pub fn get_token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    let account = svm.get_account(ata).unwrap();
    u64::from_le_bytes(account.data[64..72].try_into().unwrap())
}

// =============================================================================
// High-level setup helpers
// =============================================================================

pub fn create_mint(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    CreateMint::new(svm, payer).decimals(6).send().unwrap()
}

pub fn create_ata_and_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
    amount: u64,
) -> Pubkey {
    let ata = CreateAssociatedTokenAccount::new(svm, payer, mint)
        .owner(owner)
        .send()
        .unwrap();

    MintTo::new(svm, payer, mint, &ata, amount).send().unwrap();

    ata
}

pub fn create_test_schedule(
    svm: &mut LiteSVM,
    authority: &Keypair,
    mint: &Pubkey,
    start_time: i64,
    cliff_time: i64,
    step_duration: i64,
    total_vesting_time: i64,
    seed: u64,
) -> (Pubkey, Pubkey) {
    let auth_pk = authority.pubkey();
    let (schedule_pda, schedule_bump) = find_schedule_pda(seed, mint);
    let schedule_ata = find_ata(&schedule_pda, mint);

    let ix = build_create_schedule_ix(
        &auth_pk,
        mint,
        &schedule_pda,
        &schedule_ata,
        start_time,
        cliff_time,
        step_duration,
        total_vesting_time,
        seed,
        schedule_bump,
    );
    send_tx(svm, &[ix], &[authority]);
    (schedule_pda, schedule_ata)
}

pub struct VestingSetup {
    pub schedule_pda: Pubkey,
    pub schedule_ata: Pubkey,
    pub allocation_pda: Pubkey,
    pub authority: Keypair,
    pub recipient: Keypair,
    pub recipient_ata: Pubkey,
    pub mint: Pubkey,
}

pub fn setup_vesting_scenario(
    start_time: i64,
    cliff_time: i64,
    step_duration: i64,
    total_vesting_time: i64,
    amount: u64,
) -> (LiteSVM, VestingSetup) {
    let (mut svm, authority) = setup();
    let mint = create_mint(&mut svm, &authority);

    let seed: u64 = 42;
    let (schedule_pda, schedule_ata) = create_test_schedule(
        &mut svm,
        &authority,
        &mint,
        start_time,
        cliff_time,
        step_duration,
        total_vesting_time,
        seed,
    );

    let auth_pk = authority.pubkey();
    let _authority_ata = create_ata_and_mint(&mut svm, &authority, &mint, &auth_pk, amount);

    let recipient = Keypair::new();
    svm.airdrop(&recipient.pubkey(), 10_000_000_000).unwrap();

    let (allocation_pda, allocation_bump) = find_allocation_pda(&recipient.pubkey(), &schedule_pda);

    let ix = build_create_allocation_ix(
        &auth_pk,
        &_authority_ata,
        &schedule_pda,
        &schedule_ata,
        &allocation_pda,
        &recipient.pubkey(),
        amount,
        allocation_bump,
    );
    send_tx(&mut svm, &[ix], &[&authority]);

    // Create recipient ATA (empty)
    let recipient_ata = CreateAssociatedTokenAccount::new(&mut svm, &authority, &mint)
        .owner(&recipient.pubkey())
        .send()
        .unwrap();

    let vs = VestingSetup {
        schedule_pda,
        schedule_ata,
        allocation_pda,
        authority,
        recipient,
        recipient_ata,
        mint,
    };

    (svm, vs)
}
