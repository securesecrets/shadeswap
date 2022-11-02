use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shadeswap_shared::snip20::InitialBalance;
use cosmwasm_std::{Addr, Uint128, Binary };

/// This type represents optional configuration values which can be overridden.
/// All values are optional and have defaults which are more private by default,
/// but can be overridden if necessary
#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    pub public_total_supply: Option<bool>,
    /// Indicates whether deposit functionality should be enabled
    /// default: False
    pub enable_deposit: Option<bool>,
    /// Indicates whether redeem functionality should be enabled
    /// default: False
    pub enable_redeem: Option<bool>,
    /// Indicates whether mint functionality should be enabled
    /// default: False
    pub enable_mint: Option<bool>,
    /// Indicates whether burn functionality should be enabled
    /// default: False
    pub enable_burn: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct InitialAllowance {
    pub owner: Addr,
    pub spender: Addr,
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
}

impl InitMsg {
    pub fn config(&self) -> InitConfig {
        self.config.clone().unwrap_or_default()
    }
}
