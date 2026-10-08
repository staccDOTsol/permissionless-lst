use solana_sdk::{hash::Hash, instruction::Instruction, pubkey::Pubkey, signer::signers::Signers, transaction::Transaction};
pub fn transaction<T: Signers + ?Sized>(ixs: &[Instruction], payer: Option<&Pubkey>, signers: &T, hash: Hash) -> Transaction {
    let ixs: Vec<_> = ixs.iter().cloned().map(|mut ix| {
        if std::env::var_os("ENGINE_S_NAMESPACE").is_some() && ix.program_id == s_controller_lib::program::ID {
            ix.data.insert(0, 37);
        }
        ix
    }).collect();
    Transaction::new_signed_with_payer(&ixs, payer, signers, hash)
}
