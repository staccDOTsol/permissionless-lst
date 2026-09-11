//! Isolated accounting tests. The value calculator is a 1:1 test double;
//! controller, Token-2022, SPL Token and transfer-fee execution are real processors.
use s_controller_interface::LstState;
use s_controller_lib::*;
use s_controller_test_utils::{PoolStateProgramTest, DEFAULT_POOL_STATE};
use sanctum_token_lib::MintWithTokenProgram;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::set_return_data, pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{
        transfer_fee::{TransferFeeAmount, TransferFeeConfig},
        ExtensionType, StateWithExtensionsMut,
    },
    state::{Account as TokenAccount, AccountState, Mint},
};
const AMOUNT: u64 = 1_000_000;
fn calc(_: &Pubkey, _: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
    let mut out = amount.to_le_bytes().to_vec();
    out.extend_from_slice(&amount.to_le_bytes());
    set_return_data(&out);
    Ok(())
}
fn acc(owner: Pubkey, data: Vec<u8>) -> Account {
    Account {
        lamports: 10_000_000,
        data,
        owner,
        executable: false,
        rent_epoch: 0,
    }
}
fn mint(tp: Pubkey) -> Account {
    let ext = if tp == spl_token_2022::ID {
        vec![ExtensionType::TransferFeeConfig]
    } else {
        vec![]
    };
    let mut data = vec![0; ExtensionType::try_calculate_account_len::<Mint>(&ext).unwrap()];
    let mut m = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
    m.base.is_initialized = true;
    m.base.decimals = 9;
    m.base.supply = 20_000_000_000;
    m.pack_base();
    if tp == spl_token_2022::ID {
        let f = m.init_extension::<TransferFeeConfig>(true).unwrap();
        f.older_transfer_fee.transfer_fee_basis_points = 50u16.into();
        f.older_transfer_fee.maximum_fee = u64::MAX.into();
        f.newer_transfer_fee = f.older_transfer_fee;
    }
    m.init_account_type().unwrap();
    acc(tp, data)
}
fn token(tp: Pubkey, mint: Pubkey, owner: Pubkey, amount: u64) -> Account {
    let ext = if tp == spl_token_2022::ID {
        vec![ExtensionType::TransferFeeAmount]
    } else {
        vec![]
    };
    let mut data = vec![0; ExtensionType::try_calculate_account_len::<TokenAccount>(&ext).unwrap()];
    let mut t = StateWithExtensionsMut::<TokenAccount>::unpack_uninitialized(&mut data).unwrap();
    t.base.mint = mint;
    t.base.owner = owner;
    t.base.amount = amount;
    t.base.state = AccountState::Initialized;
    t.pack_base();
    if tp == spl_token_2022::ID {
        t.init_extension::<TransferFeeAmount>(true).unwrap();
    }
    t.init_account_type().unwrap();
    acc(tp, data)
}
async fn run(exact_out: bool, too_high_minimum: bool, lp_roundtrip: bool) {
    let input = Pubkey::new_unique();
    let output = Pubkey::new_unique();
    let calculator = if cfg!(feature = "permissionless") {
        solana_program::pubkey!("sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy")
    } else {
        Pubkey::new_unique()
    };
    let wallet = Keypair::new();
    let source = Pubkey::new_unique();
    let dest = Pubkey::new_unique();
    let mut pt = ProgramTest::new(
        "s_controller",
        program::ID,
        processor!(s_controller::entrypoint::process_instruction),
    );
    pt.prefer_bpf(false); // With BPF_OUT_DIR set, only the controller uses the SBF artifact.
    pt.add_program(
        "token",
        spl_token::ID,
        processor!(spl_token::processor::Processor::process),
    );
    pt.add_program(
        "token22",
        spl_token_2022::ID,
        processor!(spl_token_2022::processor::Processor::process),
    );
    if !cfg!(feature = "permissionless") {
        pt.add_program("calc", calculator, processor!(calc));
    }
    pt.add_program(
        "pricing",
        no_fee_pricing_program::ID,
        processor!(no_fee_pricing_program::process_instruction),
    );
    let lp_mint = Pubkey::new_unique();
    let lp_account = Pubkey::new_unique();
    let mut state = DEFAULT_POOL_STATE;
    state.lp_token_mint = lp_mint;
    state.pricing_program = no_fee_pricing_program::ID;
    state.total_sol_value = 20_000_000_000;
    pt = pt.add_pool_state(state);
    let mut list = vec![];
    let mut reserves = vec![];
    let backing_pools = [Pubkey::new_unique(), Pubkey::new_unique()];
    for mint_key in [input, output] {
        let tp = spl_token_2022::ID;
        let keys = FindLstPdaAtaKeys {
            lst_mint: mint_key,
            token_program: tp,
        };
        let (reserve, rb) = find_pool_reserves_address(keys);
        let (fee, fb) = find_protocol_fee_accumulator_address(keys);
        let mut mint_acc = mint(tp);
        if cfg!(feature = "permissionless") {
            let backing = backing_pools[list.len()];
            let sp = solana_program::pubkey!("SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY");
            let auth = Pubkey::find_program_address(&[backing.as_ref(), b"withdraw"], &sp).0;
            mint_acc.data[0..4].copy_from_slice(&1u32.to_le_bytes());
            mint_acc.data[4..36].copy_from_slice(auth.as_ref());
            let mut pool = vec![0; 611];
            pool[0] = 1;
            pool[162..194].copy_from_slice(mint_key.as_ref());
            pool[226..258].copy_from_slice(tp.as_ref());
            pool[258..266].copy_from_slice(&20_000_000_000u64.to_le_bytes());
            pool[266..274].copy_from_slice(&20_000_000_000u64.to_le_bytes());
            pt.add_account(backing, acc(sp, pool));
        }
        pt.add_account(mint_key, mint_acc);
        pt.add_account(
            reserve,
            token(tp, mint_key, program::POOL_STATE_ID, 10_000_000_000),
        );
        pt.add_account(fee, token(tp, mint_key, program::PROTOCOL_FEE_ID, 0));
        list.push(LstState {
            mint: mint_key,
            sol_value_calculator: calculator,
            pool_reserves_bump: rb,
            protocol_fee_accumulator_bump: fb,
            sol_value: 10_000_000_000,
            is_input_disabled: 0,
            padding: [0; 5],
        });
        reserves.push(reserve);
    }
    let list_data = bytemuck::cast_slice(&list).to_vec();
    pt.add_account(
        program::LST_STATE_LIST_ID,
        acc(program::ID, list_data.clone()),
    );
    pt.add_account(
        source,
        token(spl_token_2022::ID, input, wallet.pubkey(), 2 * AMOUNT),
    );
    pt.add_account(dest, token(spl_token_2022::ID, output, wallet.pubkey(), 0));
    let mut lp_mint_acc = mint(spl_token::ID);
    lp_mint_acc.data[0..4].copy_from_slice(&1u32.to_le_bytes());
    lp_mint_acc.data[4..36].copy_from_slice(program::POOL_STATE_ID.as_ref());
    pt.add_account(lp_mint, lp_mint_acc);
    pt.add_account(
        lp_account,
        token(spl_token::ID, lp_mint, wallet.pubkey(), 0),
    );
    let mut ctx = pt.start_with_context().await;
    let free = SwapByMintsFreeArgs {
        signer: wallet.pubkey(),
        src_lst_acc: source,
        dst_lst_acc: dest,
        src_lst_mint: MintWithTokenProgram {
            pubkey: input,
            token_program: spl_token_2022::ID,
        },
        dst_lst_mint: MintWithTokenProgram {
            pubkey: output,
            token_program: spl_token_2022::ID,
        },
        lst_state_list: acc(program::ID, list_data),
    };
    let mut src_calc = vec![solana_program::instruction::AccountMeta::new_readonly(
        input, false,
    )];
    let mut dst_calc = vec![solana_program::instruction::AccountMeta::new_readonly(
        output, false,
    )];
    if cfg!(feature = "permissionless") {
        src_calc.push(solana_program::instruction::AccountMeta::new_readonly(
            backing_pools[0],
            false,
        ));
        dst_calc.push(solana_program::instruction::AccountMeta::new_readonly(
            backing_pools[1],
            false,
        ));
    }
    let suffix = SrcDstLstSolValueCalcAccountSuffixes {
        src_lst_calculator_accounts: &src_calc,
        dst_lst_calculator_accounts: &dst_calc,
    };
    let pricing = [
        solana_program::instruction::AccountMeta::new_readonly(input, false),
        solana_program::instruction::AccountMeta::new_readonly(output, false),
    ];
    let net = |a: u64| a - (a * 50 + 9999) / 10000;
    let gross = |a: u64| (a * 10000 + 9949) / 9950;
    let expected = if exact_out { AMOUNT } else { net(net(AMOUNT)) };
    let spent = if exact_out {
        gross(gross(AMOUNT) + u64::from(cfg!(feature = "permissionless")))
    } else {
        AMOUNT
    };
    if lp_roundtrip {
        let pool_account = acc(program::ID, bytemuck::bytes_of(&state).to_vec());
        let lst_account = free.lst_state_list.clone();
        let mint_info = MintWithTokenProgram {
            pubkey: input,
            token_program: spl_token_2022::ID,
        };
        let lp_suffix = AddRemoveLiquidityAccountSuffixes {
            lst_calculator_accounts: &src_calc,
            pricing_program_price_lp_accounts: &pricing[..1],
        };
        let add = add_liquidity_ix_by_mint_full(
            AddLiquidityByMintFreeArgs {
                signer: wallet.pubkey(),
                src_lst_acc: source,
                dst_lp_acc: lp_account,
                pool_state: pool_account.clone(),
                lst_state_list: lst_account.clone(),
                lst_mint: mint_info,
            },
            AddLiquidityIxAmts {
                lst_amount: AMOUNT,
                min_lp_out: 995_000,
            },
            lp_suffix,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[add],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &wallet],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
        let balance = |a: Account| u64::from_le_bytes(a.data[64..72].try_into().unwrap());
        assert_eq!(
            balance(
                ctx.banks_client
                    .get_account(lp_account)
                    .await
                    .unwrap()
                    .unwrap()
            ),
            995_000
        );
        let remove = remove_liquidity_ix_by_mint_full(
            RemoveLiquidityByMintFreeArgs {
                signer: wallet.pubkey(),
                src_lp_acc: lp_account,
                dst_lst_acc: source,
                pool_state: pool_account,
                lst_state_list: lst_account,
                lst_mint: mint_info,
            },
            RemoveLiquidityIxAmts {
                lp_token_amount: 995_000,
                min_lst_out: 990_025,
            },
            lp_suffix,
        )
        .unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[remove],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &wallet],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
        assert_eq!(
            balance(
                ctx.banks_client
                    .get_account(lp_account)
                    .await
                    .unwrap()
                    .unwrap()
            ),
            0
        );
        assert_eq!(
            balance(ctx.banks_client.get_account(source).await.unwrap().unwrap()),
            1_990_025
        );
        assert_eq!(
            balance(
                ctx.banks_client
                    .get_account(reserves[0])
                    .await
                    .unwrap()
                    .unwrap()
            ),
            10_000_000_000
        );
        return;
    }
    let ix = if exact_out {
        swap_exact_out_ix_by_mint_full(
            free,
            SwapExactOutAmounts {
                max_amount_in: spent,
                amount: AMOUNT,
            },
            suffix,
            &pricing,
            no_fee_pricing_program::ID,
        )
        .unwrap()
    } else {
        swap_exact_in_ix_by_mint_full(
            free,
            SwapExactInAmounts {
                amount: AMOUNT,
                min_amount_out: if too_high_minimum {
                    expected + 1
                } else {
                    expected
                },
            },
            suffix,
            &pricing,
            no_fee_pricing_program::ID,
        )
        .unwrap()
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &wallet],
        ctx.last_blockhash,
    );
    let result = ctx.banks_client.process_transaction(tx).await;
    if too_high_minimum {
        assert_eq!(
            result.unwrap_err().unwrap(),
            solana_sdk::transaction::TransactionError::InstructionError(
                0,
                solana_sdk::instruction::InstructionError::Custom(
                    s_controller_interface::SControllerError::SlippageToleranceExceeded as u32
                )
            )
        );
    } else {
        result.unwrap();
    }
    let get_amount = |a: Account| u64::from_le_bytes(a.data[64..72].try_into().unwrap());
    assert_eq!(
        get_amount(ctx.banks_client.get_account(dest).await.unwrap().unwrap()),
        if too_high_minimum { 0 } else { expected }
    );
    assert_eq!(
        get_amount(ctx.banks_client.get_account(source).await.unwrap().unwrap()),
        if too_high_minimum {
            2 * AMOUNT
        } else {
            2 * AMOUNT - spent
        }
    );
}
#[tokio::test]
async fn exact_in_both_legs_charge_transfer_fees() {
    run(false, false, false).await;
}
#[tokio::test]
async fn exact_out_both_legs_gross_up_transfer_fees() {
    run(true, false, false).await;
}
#[tokio::test]
async fn minimum_is_net_and_failed_swap_rolls_back() {
    run(false, true, false).await;
}

#[tokio::test]
async fn lp_roundtrip_credits_only_net_deposit_and_net_redemption() {
    run(false, false, true).await;
}
