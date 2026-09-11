use pricing_programs_interface::{
    PriceExactInIxArgs, PriceExactInKeys, PriceExactOutIxArgs, PriceExactOutKeys,
    PriceLpTokensToMintIxArgs, PriceLpTokensToRedeemIxArgs,
};
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use solana_readonly_account::ReadonlyAccountData;
use std::collections::HashMap;

mod err;
mod flat_fee;
mod traits;

pub use err::*;
pub use flat_fee::*;
pub use traits::*;

#[derive(Clone, Debug)]
pub enum KnownPricingProg {
    FlatFee(FlatFeePricingProg),
    BuiltinNoFee, // eat.ag permissionless controller; no CPI or mutable fee table
}

impl MutablePricingProg for KnownPricingProg {
    fn try_new<I: Iterator<Item = Pubkey>>(
        program_id: Pubkey,
        mints: I,
    ) -> Result<Self, PricingProgErr> {
        Ok(match program_id {
            flat_fee_lib::program::ID => {
                Self::FlatFee(FlatFeePricingProg::try_new(program_id, mints)?)
            }
            x if x == solana_program::pubkey!("NoFEEPR1C1NGPRoGRAM111111111111111111111111") => {
                Self::BuiltinNoFee
            }
            _ => Err(PricingProgErr::UnknownPricingProg)?,
        })
    }

    fn get_accounts_to_update_for_all_lsts(&self) -> Vec<Pubkey> {
        match self {
            Self::BuiltinNoFee => Vec::new(),
            Self::FlatFee(p) => p.get_accounts_to_update_for_all_lsts(),
        }
    }

    fn get_accounts_to_update_for_liquidity(&self) -> Vec<Pubkey> {
        match self {
            Self::BuiltinNoFee => Vec::new(),
            Self::FlatFee(p) => p.get_accounts_to_update_for_liquidity(),
        }
    }

    fn get_accounts_to_update_for_lsts<I: Iterator<Item = Pubkey>>(
        &self,
        lst_mints: I,
    ) -> Vec<Pubkey> {
        match self {
            Self::BuiltinNoFee => Vec::new(),
            Self::FlatFee(p) => p.get_accounts_to_update_for_lsts(lst_mints),
        }
    }

    fn update<D: ReadonlyAccountData>(
        &mut self,
        account_map: &HashMap<Pubkey, D>,
    ) -> anyhow::Result<()> {
        match self {
            Self::BuiltinNoFee => Ok(()),
            Self::FlatFee(p) => p.update(account_map),
        }
    }
}

impl PricingProg for KnownPricingProg {
    fn pricing_program_id(&self) -> Pubkey {
        match self {
            Self::BuiltinNoFee => {
                solana_program::pubkey!("NoFEEPR1C1NGPRoGRAM111111111111111111111111")
            }
            Self::FlatFee(p) => p.pricing_program_id(),
        }
    }

    fn quote_lp_tokens_to_redeem(
        &self,
        output_lst_mint: Pubkey,
        args: &PriceLpTokensToRedeemIxArgs,
    ) -> anyhow::Result<u64> {
        match self {
            Self::BuiltinNoFee => Ok(args.sol_value),
            Self::FlatFee(p) => p.quote_lp_tokens_to_redeem(output_lst_mint, args),
        }
    }

    fn price_lp_tokens_to_redeem_accounts(
        &self,
        output_lst_mint: Pubkey,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        match self {
            Self::BuiltinNoFee => Ok(vec![AccountMeta::new_readonly(output_lst_mint, false)]),
            Self::FlatFee(p) => p.price_lp_tokens_to_redeem_accounts(output_lst_mint),
        }
    }

    fn quote_lp_tokens_to_mint(
        &self,
        input_lst_mint: Pubkey,
        args: &PriceLpTokensToMintIxArgs,
    ) -> anyhow::Result<u64> {
        match self {
            Self::BuiltinNoFee => Ok(args.sol_value),
            Self::FlatFee(p) => p.quote_lp_tokens_to_mint(input_lst_mint, args),
        }
    }

    fn price_lp_tokens_to_mint_accounts(
        &self,
        input_lst_mint: Pubkey,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        match self {
            Self::BuiltinNoFee => Ok(vec![AccountMeta::new_readonly(input_lst_mint, false)]),
            Self::FlatFee(p) => p.price_lp_tokens_to_mint_accounts(input_lst_mint),
        }
    }

    fn quote_exact_in(
        &self,
        keys: PriceExactInKeys,
        args: &PriceExactInIxArgs,
    ) -> anyhow::Result<u64> {
        match self {
            Self::BuiltinNoFee => Ok(args.sol_value),
            Self::FlatFee(p) => p.quote_exact_in(keys, args),
        }
    }

    fn price_exact_in_accounts(&self, keys: PriceExactInKeys) -> anyhow::Result<Vec<AccountMeta>> {
        match self {
            Self::BuiltinNoFee => Ok(vec![
                AccountMeta::new_readonly(keys.input_lst_mint, false),
                AccountMeta::new_readonly(keys.output_lst_mint, false),
            ]),
            Self::FlatFee(p) => p.price_exact_in_accounts(keys),
        }
    }

    fn quote_exact_out(
        &self,
        keys: PriceExactOutKeys,
        args: &PriceExactOutIxArgs,
    ) -> anyhow::Result<u64> {
        match self {
            Self::BuiltinNoFee => Ok(args.sol_value),
            Self::FlatFee(p) => p.quote_exact_out(keys, args),
        }
    }

    fn price_exact_out_accounts(
        &self,
        keys: PriceExactOutKeys,
    ) -> anyhow::Result<Vec<AccountMeta>> {
        match self {
            Self::BuiltinNoFee => Ok(vec![
                AccountMeta::new_readonly(keys.input_lst_mint, false),
                AccountMeta::new_readonly(keys.output_lst_mint, false),
            ]),
            Self::FlatFee(p) => p.price_exact_out_accounts(keys),
        }
    }
}

impl From<FlatFeePricingProg> for KnownPricingProg {
    fn from(value: FlatFeePricingProg) -> Self {
        Self::FlatFee(value)
    }
}
