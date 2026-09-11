//! eat.ag: remove curator admission, retain independently verifiable value provenance.
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_option::COption, pubkey,
    pubkey::Pubkey,
};
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    state::Mint,
};
const SP12: Pubkey = pubkey!("SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY");
const CALC: Pubkey = pubkey!("sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy");
pub fn verify_backing(
    mint: &AccountInfo,
    calc: &AccountInfo,
    pool: Option<&AccountInfo>,
) -> Result<(), ProgramError> {
    sanctum_s_common::token::verify_tokenkeg_or_22_mint(mint)?;
    if *mint.key == spl_token::native_mint::ID
        && *mint.owner == spl_token::ID
        && *calc.key == pubkey!("wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE")
    {
        return Ok(());
    }
    // Known ABIs are selected by the backing program, not by a curated token list.
    // Arbitrary price-reporting executables cannot introduce counterfeit SOL backing.
    let pool = pool.ok_or(ProgramError::NotEnoughAccountKeys)?;
    let expected_calc = match *pool.owner {
        SP12 => CALC,
        x if x == pubkey!("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy") => {
            pubkey!("sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF")
        }
        x if x == pubkey!("SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn") => {
            pubkey!("ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo")
        }
        _ => return Err(ProgramError::InvalidAccountData),
    };
    if *calc.key != expected_calc {
        return Err(ProgramError::InvalidAccountData);
    }
    let data = pool.try_borrow_data()?;
    if data.len() < 282
        || data[0] != 1
        || &data[162..194] != mint.key.as_ref()
        || &data[226..258] != mint.owner.as_ref()
    {
        return Err(ProgramError::InvalidAccountData);
    }
    let mint_data = mint.try_borrow_data()?;
    let m = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    // Route admission is public, but reserves must remain transferable and cannot
    // be seized by a permanent delegate or controlled by an arbitrary hook.
    if m.get_extension_types()?.iter().any(|ext| {
        !matches!(
            ext,
            ExtensionType::TransferFeeConfig
                | ExtensionType::MintCloseAuthority
                | ExtensionType::MetadataPointer
                | ExtensionType::TokenMetadata
        )
    }) {
        return Err(ProgramError::InvalidAccountData);
    }
    let authority = Pubkey::find_program_address(&[pool.key.as_ref(), b"withdraw"], pool.owner).0;
    if m.base.decimals != 9
        || m.base.mint_authority != COption::Some(authority)
        || m.base.freeze_authority.is_some()
    {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Valuation is compiled into this fork. The calculator address is an ABI tag,
/// not an executable called by CPI; no manager-approved upgrade slot is read.
pub fn calculate_value(
    mint: &AccountInfo,
    calculator: &AccountInfo,
    pool: Option<&AccountInfo>,
    amount: u64,
    to_sol: bool,
) -> Result<sanctum_token_ratio::U64ValueRange, ProgramError> {
    use sol_value_calculator_lib::SolValueCalculator;
    use solana_program::{clock::Clock, sysvar::Sysvar};
    verify_backing(mint, calculator, pool)?;
    if *mint.key == spl_token::native_mint::ID {
        return Ok(sanctum_token_ratio::U64ValueRange::single(amount));
    }
    let pool = pool.ok_or(ProgramError::NotEnoughAccountKeys)?;
    let state = spl_calculator_lib::deserialize_stake_pool_checked(pool)?;
    let calc = spl_calculator_lib::SplStakePoolCalc::from(state);
    calc.verify_pool_updated_for_this_epoch(Clock::get()?.epoch)?;
    if to_sol {
        calc.calc_lst_to_sol(amount)
    } else {
        calc.calc_sol_to_lst(amount)
    }
}
