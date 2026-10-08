use s_controller_interface::{SControllerError, SControllerProgramIx};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::{PrintProgramError, ProgramError},
    pubkey::Pubkey,
};

use crate::processor::*;

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if *program_id != s_controller_lib::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // eat.ag extension: keep upstream discriminators intact.
    if instruction_data == [23] {
        return process_create_fee_account(program_id, accounts);
    }
    let ix = SControllerProgramIx::deserialize(instruction_data)?;
    #[cfg(feature = "permissionless")]
    if matches!(
        ix,
        SControllerProgramIx::DisableLstInput(_)
            | SControllerProgramIx::EnableLstInput(_)
            | SControllerProgramIx::SetSolValueCalculator(_)
            | SControllerProgramIx::SetPricingProgram
            | SControllerProgramIx::AddDisablePoolAuthority
            | SControllerProgramIx::RemoveDisablePoolAuthority(_)
            | SControllerProgramIx::DisablePool
            | SControllerProgramIx::EnablePool
            | SControllerProgramIx::StartRebalance(_)
            | SControllerProgramIx::EndRebalance
            | SControllerProgramIx::SetRebalanceAuthority
            | SControllerProgramIx::RemoveLst(_)
    ) {
        return Err(ProgramError::InvalidInstructionData);
    }
    solana_program::msg!("S instruction {}", instruction_data[0]);

    let res = match ix {
        SControllerProgramIx::SyncSolValue(args) => process_sync_sol_value(accounts, args),
        SControllerProgramIx::SwapExactIn(args) => process_swap_exact_in(accounts, args),
        SControllerProgramIx::SwapExactOut(args) => process_swap_exact_out(accounts, args),
        SControllerProgramIx::AddLiquidity(args) => process_add_liquidity(accounts, args),
        SControllerProgramIx::RemoveLiquidity(args) => process_remove_liquidity(accounts, args),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::DisableLstInput(args) => process_disable_lst_input(accounts, args),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::EnableLstInput(args) => process_enable_lst_input(accounts, args),
        SControllerProgramIx::AddLst => process_add_lst(accounts),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::RemoveLst(args) => process_remove_lst(accounts, args),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::SetSolValueCalculator(args) => {
            process_set_sol_value_calculator(accounts, args)
        }
        SControllerProgramIx::SetAdmin => process_set_admin(accounts),
        SControllerProgramIx::SetProtocolFee(args) => process_set_protocol_fee(accounts, args),
        SControllerProgramIx::SetProtocolFeeBeneficiary => {
            process_set_protocol_fee_beneficiary(accounts)
        }
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::SetPricingProgram => process_set_pricing_program(accounts),
        SControllerProgramIx::WithdrawProtocolFees(args) => {
            process_withdraw_protocol_fees(accounts, args)
        }
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::AddDisablePoolAuthority => {
            process_add_disable_pool_authority(accounts)
        }
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::RemoveDisablePoolAuthority(args) => {
            process_remove_disable_pool_authority(accounts, args)
        }
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::DisablePool => process_disable_pool(accounts),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::EnablePool => process_enable_pool(accounts),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::StartRebalance(args) => process_start_rebalance(accounts, args),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::EndRebalance => process_end_rebalance(accounts),
        #[cfg(not(feature = "permissionless"))]
        SControllerProgramIx::SetRebalanceAuthority => process_set_rebalance_authority(accounts),
        SControllerProgramIx::Initialize => process_initialize(accounts),
        #[cfg(feature = "permissionless")]
        _ => Err(ProgramError::InvalidInstructionData),
    };
    if let Err(e) = res.as_ref() {
        e.print::<SControllerError>();
    }
    res
}
