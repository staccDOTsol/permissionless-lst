#![allow(non_local_definitions)]

#[cfg(not(feature = "permissionless"))]
solana_program::declare_id!("5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx");
pub mod typedefs;
pub use typedefs::*;
pub mod instructions;
pub use instructions::*;
pub mod errors;
pub use errors::*;

#[cfg(feature = "permissionless")]
solana_program::declare_id!("GSsMfxpbN3h7jwkhJkbZPErjFo22a6MAgr6npcp56KZc");
