//! eat.ag fork extension. Matches controller opcode 23.
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
pub fn create_fee_account_ix_for_program(
    program_id: Pubkey,
    payer: Pubkey,
    mint: Pubkey,
    token_program: Pubkey,
) -> Instruction {
    let authority = crate::find_protocol_fee_address(program_id).0;
    let ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &authority,
        &mint,
        &token_program,
    );
    Instruction {
        program_id,
        data: vec![23],
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    }
}

/// The permissionless AddLst layout preserves upstream account order. Only the
/// payer signs. The optional appended stake pool is the onchain backing proof.
pub fn permissionless_add_lst_ix(
    keys: s_controller_interface::AddLstKeys,
    backing_pool: Option<Pubkey>,
) -> Result<Instruction, std::io::Error> {
    let mut ix = s_controller_interface::add_lst_ix(keys)?;
    ix.accounts[0].is_signer = false;
    if let Some(pool) = backing_pool {
        ix.accounts.push(AccountMeta::new_readonly(pool, false));
    }
    Ok(ix)
}
