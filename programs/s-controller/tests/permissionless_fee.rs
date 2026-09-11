//! eat.ag regression: real controller entrypoint, ATA and token processors.
use s_controller_lib::{create_fee_account_ix_for_program, find_protocol_fee_address, program::ID};
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Signer, transaction::Transaction};
use spl_token_2022::{
    extension::{
        transfer_fee::{TransferFeeAmount, TransferFeeConfig},
        BaseStateWithExtensions, ExtensionType, StateWithExtensions, StateWithExtensionsMut,
    },
    state::{Account as TokenAccount, Mint},
};
async fn run_fee_case(token_program: Pubkey, prefund: u64) {
    let mut test = ProgramTest::new(
        "s_controller",
        ID,
        processor!(s_controller::entrypoint::process_instruction),
    );
    test.prefer_bpf(false); // With BPF_OUT_DIR set, only the controller uses the SBF artifact.
    test.add_program(
        "spl_token",
        spl_token::ID,
        processor!(spl_token::processor::Processor::process),
    );
    test.add_program(
        "spl_token_2022",
        spl_token_2022::ID,
        processor!(spl_token_2022::processor::Processor::process),
    );
    test.add_program(
        "spl_associated_token_account",
        spl_associated_token_account::ID,
        processor!(spl_associated_token_account::processor::process_instruction),
    );
    let mint = Pubkey::new_unique();
    let authority = find_protocol_fee_address(ID).0;
    let fee = spl_associated_token_account::get_associated_token_address_with_program_id(
        &authority,
        &mint,
        &token_program,
    );
    let is22 = token_program == spl_token_2022::ID;
    let ext = if is22 {
        vec![ExtensionType::TransferFeeConfig]
    } else {
        vec![]
    };
    let mut data = vec![0; ExtensionType::try_calculate_account_len::<Mint>(&ext).unwrap()];
    let mut m = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
    m.base.is_initialized = true;
    m.base.decimals = 9;
    m.pack_base();
    if is22 {
        let f = m.init_extension::<TransferFeeConfig>(true).unwrap();
        f.older_transfer_fee.transfer_fee_basis_points = 50u16.into();
        f.older_transfer_fee.maximum_fee = u64::MAX.into();
        f.newer_transfer_fee = f.older_transfer_fee;
    }
    m.init_account_type().unwrap();
    test.add_account(
        mint,
        Account {
            lamports: 10_000_000,
            data,
            owner: token_program,
            executable: false,
            rent_epoch: 0,
        },
    );
    if prefund > 0 {
        test.add_account(
            fee,
            Account {
                lamports: prefund,
                ..Account::default()
            },
        );
    }
    let mut ctx = test.start_with_context().await;
    let ix = create_fee_account_ix_for_program(ID, ctx.payer.pubkey(), mint, token_program);
    // Random fee payer, no controller admin signer or pool state.
    let tx = Transaction::new_signed_with_payer(
        &[ix.clone()],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    let account = ctx.banks_client.get_account(fee).await.unwrap().unwrap();
    assert_eq!(account.owner, token_program);
    let t = StateWithExtensions::<TokenAccount>::unpack(&account.data).unwrap();
    assert_eq!(t.base.owner, authority);
    assert_eq!(t.base.mint, mint);
    assert_eq!(t.base.amount, 0);
    if is22 {
        assert!(t.get_extension::<TransferFeeAmount>().is_ok());
    }
    let tx = Transaction::new_signed_with_payer(
        &[ix.clone(), ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    let after = ctx.banks_client.get_account(fee).await.unwrap().unwrap();
    assert_eq!(account, after);
    let mut wrong = create_fee_account_ix_for_program(ID, ctx.payer.pubkey(), mint, token_program);
    wrong.accounts[3].pubkey = Pubkey::new_unique();
    let tx = Transaction::new_signed_with_payer(
        &[wrong],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    assert!(ctx.banks_client.process_transaction(tx).await.is_err());
}
#[tokio::test]
async fn token22_fee_ata_is_permissionless_and_idempotent() {
    run_fee_case(spl_token_2022::ID, 0).await;
}
#[tokio::test]
async fn prefunded_token22_fee_ata_is_initialized() {
    run_fee_case(spl_token_2022::ID, 2_500_000).await;
}
#[tokio::test]
async fn legacy_fee_ata_remains_supported() {
    run_fee_case(spl_token::ID, 0).await;
}
