//! TODO: stuff in here should probably be moved to sanctum-token-lib

use sanctum_token_lib::mint_supply;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::extension::StateWithExtensions;

pub fn verify_tokenkeg_or_22_mint(mint: &AccountInfo) -> Result<(), ProgramError> {
    if *mint.owner != spl_token::ID && *mint.owner != spl_token_2022::ID {
        return Err(ProgramError::IllegalOwner);
    }
    // TODO: change this to `sanctum_token_lib::ValidMintAccount::mint_is_initialized()`
    // when we upgrade `sanctum-token-lib`
    // trying to read mint.supply field verifies that the mint is initialized.
    mint_supply(mint)?;
    Ok(())
}

pub fn verify_token_account_authority(
    token_account: &AccountInfo,
    expected_authority: Pubkey,
) -> Result<(), ProgramError> {
    let StateWithExtensions { base, .. } =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(
            &token_account.try_borrow_data()?,
        )?;
    if base.owner != expected_authority {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

// eat.ag: use the token program's fee arithmetic, including caps and epoch transitions.
pub fn net_transfer_amount(mint: &AccountInfo, amount: u64) -> Result<u64, ProgramError> {
    use solana_program::{clock::Clock, sysvar::Sysvar};
    net_transfer_amount_from_data(
        mint.owner,
        &mint.try_borrow_data()?,
        Clock::get()?.epoch,
        amount,
    )
}
pub fn net_transfer_amount_from_data(
    owner: &Pubkey,
    data: &[u8],
    epoch: u64,
    amount: u64,
) -> Result<u64, ProgramError> {
    use spl_token_2022::extension::{
        transfer_fee::TransferFeeConfig, BaseStateWithExtensions, ExtensionType,
    };
    if *owner == spl_token::ID {
        return Ok(amount);
    }
    if *owner != spl_token_2022::ID {
        return Err(ProgramError::IllegalOwner);
    }
    let m = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(data)?;
    let fee = if m
        .get_extension_types()?
        .contains(&ExtensionType::TransferFeeConfig)
    {
        m.get_extension::<TransferFeeConfig>()?
            .calculate_epoch_fee(epoch, amount)
            .ok_or(ProgramError::InvalidArgument)?
    } else {
        0
    };
    amount.checked_sub(fee).ok_or(ProgramError::InvalidArgument)
}
pub fn gross_transfer_amount(mint: &AccountInfo, net: u64) -> Result<u64, ProgramError> {
    use solana_program::{clock::Clock, sysvar::Sysvar};
    gross_transfer_amount_from_data(
        mint.owner,
        &mint.try_borrow_data()?,
        Clock::get()?.epoch,
        net,
    )
}
pub fn gross_transfer_amount_from_data(
    owner: &Pubkey,
    data: &[u8],
    epoch: u64,
    net: u64,
) -> Result<u64, ProgramError> {
    use spl_token_2022::extension::{
        transfer_fee::TransferFeeConfig, BaseStateWithExtensions, ExtensionType,
    };
    if *owner == spl_token::ID {
        return Ok(net);
    }
    if *owner != spl_token_2022::ID {
        return Err(ProgramError::IllegalOwner);
    }
    let m = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(data)?;
    let fee = if m
        .get_extension_types()?
        .contains(&ExtensionType::TransferFeeConfig)
    {
        m.get_extension::<TransferFeeConfig>()?
            .calculate_inverse_epoch_fee(epoch, net)
            .ok_or(ProgramError::InvalidArgument)?
    } else {
        0
    };
    let gross = net.checked_add(fee).ok_or(ProgramError::InvalidArgument)?;
    if net_transfer_amount_from_data(owner, data, epoch, gross)? < net {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(gross)
}
