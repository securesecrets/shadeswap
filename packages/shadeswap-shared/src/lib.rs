pub mod msg;
pub use fadroma;
pub use token_pair::*;
pub use token_type::*;
pub use token_amount::*;
pub use token_pair_amount::*;
pub mod token_pair;
pub mod token_type;
pub mod token_amount;
pub mod token_pair_amount;
pub mod amm_pair;
pub mod admin;

#[cfg(not(target_arch = "wasm32"))]
pub mod querier;
mod display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use composable_snip20 as snip20_impl;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}
