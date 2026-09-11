//! eat.ag addition: initialize the correct fee ATA for any mint, without AddLst/admin.
use sanctum_s_common::token::verify_tokenkeg_or_22_mint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, system_program,
};

/// ABI 23: [payer SW, mint R, fee authority R, fee ATA W, token program R,
/// associated-token program R, system program R]. No registry entry is needed.
pub fn process_create_fee_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if accounts.len() != 7 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let [payer, mint, authority, ata, token, associated, system] = accounts else {
        unreachable!()
    };
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !payer.is_writable || !ata.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    verify_tokenkeg_or_22_mint(mint)?;
    let expected_authority = s_controller_lib::find_protocol_fee_address(*program_id).0;
    let expected_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &expected_authority,
        mint.key,
        mint.owner,
    );
    if *authority.key != expected_authority
        || *ata.key != expected_ata
        || token.key != mint.owner
        || !token.executable
        || *associated.key != spl_associated_token_account::ID
        || !associated.executable
        || *system.key != system_program::ID
        || !system.executable
    {
        return Err(ProgramError::InvalidAccountData);
    }
    // The ATA program allocates TransferFeeAmount from the mint's extensions,
    // validates initialized accounts, and handles System-owned pre-funded PDAs.
    let ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        payer.key,
        authority.key,
        mint.key,
        token.key,
    );
    invoke(
        &ix,
        &[
            payer.clone(),
            ata.clone(),
            authority.clone(),
            mint.clone(),
            system.clone(),
            token.clone(),
            associated.clone(),
        ],
    )
}
