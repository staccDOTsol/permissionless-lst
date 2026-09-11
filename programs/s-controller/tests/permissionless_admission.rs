//! Admission executes the controller and ATA/Token-2022 programs. Backing pool
//! bytes model the SPL ABI; no curator signs and no external calculator is called.
#![cfg(feature = "permissionless")]
use s_controller_interface::AddLstKeys;
use s_controller_lib::{permissionless_add_lst_ix, program, try_lst_state_list};
use s_controller_test_utils::{PoolStateProgramTest, DEFAULT_POOL_STATE};
use solana_program::{program_option::COption, pubkey, pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{account::Account, signature::Signer, transaction::Transaction};
use spl_token_2022::{
    extension::{
        transfer_fee::{TransferFeeAmount, TransferFeeConfig},
        BaseStateWithExtensions, ExtensionType, StateWithExtensions, StateWithExtensionsMut,
    },
    state::{Account as TokenAccount, Mint},
};
const SP12: Pubkey = pubkey!("SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY");
const CALCULATOR: Pubkey = pubkey!("sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy");
async fn admission(forged_pool: bool, wrong_calculator: bool, frozen_mint: bool) {
    let mint = Pubkey::new_unique();
    let backing_pool = Pubkey::new_unique();
    let mut pool_data = vec![0; 611];
    pool_data[0] = 1;
    pool_data[162..194].copy_from_slice(mint.as_ref());
    pool_data[226..258].copy_from_slice(spl_token_2022::ID.as_ref());
    let mut mint_data =
        vec![
            0;
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferFeeConfig])
                .unwrap()
        ];
    let mut m = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut mint_data).unwrap();
    m.base.is_initialized = true;
    m.base.decimals = 9;
    m.base.mint_authority =
        COption::Some(Pubkey::find_program_address(&[backing_pool.as_ref(), b"withdraw"], &SP12).0);
    if frozen_mint {
        m.base.freeze_authority = COption::Some(Pubkey::new_unique());
    }
    m.pack_base();
    m.init_extension::<TransferFeeConfig>(true).unwrap();
    m.init_account_type().unwrap();
    let mut pt = ProgramTest::new(
        "s_controller",
        program::ID,
        processor!(s_controller::entrypoint::process_instruction),
    );
    pt.prefer_bpf(false); // With BPF_OUT_DIR set, only the controller uses the SBF artifact.
    pt.add_program(
        "token22",
        spl_token_2022::ID,
        processor!(spl_token_2022::processor::Processor::process),
    );
    pt.add_program(
        "ata",
        spl_associated_token_account::ID,
        processor!(spl_associated_token_account::processor::process_instruction),
    );
    let calc = if wrong_calculator {
        Pubkey::new_unique()
    } else {
        CALCULATOR
    };

    let admin = Pubkey::new_unique();
    let mut state = DEFAULT_POOL_STATE;
    state.admin = admin;
    // A stale disabled flag cannot gate permissionless admission.
    state.is_disabled = 1;
    pt = pt.add_pool_state(state);
    pt.add_account(
        mint,
        Account {
            lamports: 10_000_000,
            data: mint_data,
            owner: spl_token_2022::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    pt.add_account(
        backing_pool,
        Account {
            lamports: 10_000_000,
            data: pool_data,
            owner: if forged_pool {
                system_program::ID
            } else {
                SP12
            },
            executable: false,
            rent_epoch: 0,
        },
    );
    let reserve = spl_associated_token_account::get_associated_token_address_with_program_id(
        &program::POOL_STATE_ID,
        &mint,
        &spl_token_2022::ID,
    );
    let fee = spl_associated_token_account::get_associated_token_address_with_program_id(
        &program::PROTOCOL_FEE_ID,
        &mint,
        &spl_token_2022::ID,
    );
    let mut ctx = pt.start_with_context().await;
    let ix = permissionless_add_lst_ix(
        AddLstKeys {
            admin,
            payer: ctx.payer.pubkey(),
            lst_mint: mint,
            pool_reserves: reserve,
            protocol_fee_accumulator: fee,
            protocol_fee_accumulator_auth: program::PROTOCOL_FEE_ID,
            sol_value_calculator: calc,
            pool_state: program::POOL_STATE_ID,
            lst_state_list: program::LST_STATE_LIST_ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: system_program::ID,
            lst_token_program: spl_token_2022::ID,
        },
        Some(backing_pool),
    )
    .unwrap();
    assert!(!ix.accounts[0].is_signer);
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    let result = ctx.banks_client.process_transaction(tx).await;
    if forged_pool || wrong_calculator || frozen_mint {
        assert_eq!(
            result.unwrap_err().unwrap(),
            solana_sdk::transaction::TransactionError::InstructionError(
                0,
                solana_sdk::instruction::InstructionError::InvalidAccountData
            )
        );
        assert!(ctx.banks_client.get_account(fee).await.unwrap().is_none());
        return;
    }
    result.unwrap();
    for address in [reserve, fee] {
        let account = ctx
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.owner, spl_token_2022::ID);
        let token = StateWithExtensions::<TokenAccount>::unpack(&account.data).unwrap();
        assert_eq!(token.base.mint, mint);
        assert!(token.get_extension::<TransferFeeAmount>().is_ok());
    }
    let account = ctx
        .banks_client
        .get_account(program::LST_STATE_LIST_ID)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(try_lst_state_list(&account.data).unwrap()[0].mint, mint);
}
#[tokio::test]
async fn arbitrary_payer_lists_backed_token22_and_creates_both_vaults() {
    admission(false, false, false).await;
}
#[tokio::test]
async fn spoofed_stake_pool_owner_is_rejected() {
    admission(true, false, false).await;
}
#[tokio::test]
async fn arbitrary_price_program_is_rejected() {
    admission(false, true, false).await;
}

#[tokio::test]
async fn freeze_authority_cannot_enter_reserves() {
    admission(false, false, true).await;
}
