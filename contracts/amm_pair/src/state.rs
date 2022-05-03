use shadeswap_shared::{
    fadroma::{
        scrt_link::{ContractLink},    
        scrt_addr::{Humanize, Canonize},
        scrt::{
            Api, CanonicalAddr, Extern, HumanAddr, Uint128,
            Querier, StdResult, Storage, StdError
        },
        scrt_storage::{load, save},
        scrt_vk::ViewingKey,
    },
    token_pair::TokenPair
};

use serde::{Serialize,Deserialize};
use tradehistory::TradeHistory;
pub static CONFIG_KEY: &[u8] = b"config";
const TRADE_COUNT : &[u8] = b"trade_count";
const TRADE_HISTORY: &[u8] = b"trade_history_";
pub const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct Config<A: Clone> {
    pub symbol:        String,
    pub factory_info:  ContractLink<A>,
    pub lp_token_info: ContractLink<A>,
    pub pair:      TokenPair<A>,
    pub contract_addr: A,
    pub viewing_key: ViewingKey,
}


impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            symbol:        self.symbol.to_string(),
            factory_info:  self.factory_info.canonize(api)?,
            lp_token_info: self.lp_token_info.canonize(api)?,
            pair:          self.pair.canonize(api)?,
            contract_addr: self.contract_addr.canonize(api)?,
            viewing_key:   self.viewing_key.clone(),
        })
    }
}
impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        let trades: Vec<TradeHistory> = Vec::new();
        Ok(Config {
            symbol:        self.symbol.to_string(),
            factory_info:  self.factory_info.humanize(api)?,
            lp_token_info: self.lp_token_info.humanize(api)?,
            pair:      self.pair.humanize(api)?,
            contract_addr: self.contract_addr.humanize(api)?,
            viewing_key:   self.viewing_key.clone(),
        })
    }
}


pub(crate) fn store_config <S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config<HumanAddr>> {
    let result: Config<CanonicalAddr> = load(&deps.storage, CONFIG_KEY)?.ok_or(
        StdError::generic_err("Config doesn't exist in storage.")
    )?;
    result.humanize(&deps.api)
}

pub mod tradehistory{
    use super::*;
    
    #[derive(Serialize, Deserialize,  PartialEq, Debug, Clone)]
    pub enum DirectionType{
        Buy,
        Sell,
        Unknown,
    }

    #[derive(Serialize, Deserialize,  PartialEq, Debug, Clone)]
    pub struct TradeHistory {
        pub price: Uint128,
        pub amount: Uint128,
        pub timestamp: u64,
        pub direction: DirectionType,
    }
}


pub mod swapdetails {
    use super::*;

    #[derive(Serialize, Deserialize,  PartialEq, Debug)]
    pub struct SwapInfo {
        pub total_fee_amount: Uint128,
        pub lp_fee_amount: Uint128,
        pub shade_dao_fee_amount: Uint128,
        pub result: SwapResult,
    }
    
    #[derive(Serialize, Deserialize,  PartialEq, Debug)]
    pub struct SwapResult {
        pub return_amount: Uint128,
        pub spread_amount: Uint128,
    }
    
}

pub fn load_trade_counter(storage: &impl Storage) -> StdResult<u64> {
    Ok(load(storage, TRADE_COUNT)?.unwrap_or(0))
}

pub fn store_trade_counter(storage: &mut impl Storage, count: u64) -> StdResult<()> {
    save(storage, TRADE_COUNT, &count)
}

pub fn load_trade_history(storage: &impl Storage) -> StdResult<TradeHistory> {
    Ok(load(storage, TRADE_HISTORY)?.unwrap_or(TradeHistory{amount :Uint128::zero(), price: Uint128::zero(), direction: DirectionType::Unknown, timestamp: 0 }))
}

pub fn store_trade_history(storage: &mut impl Storage, trade_history: TradeHistory, count: u64) -> StdResult<()> {
    let new_store_at: &[u8] = "trade_history_" + count.to_string();    
    save(storage, new_store_at, &trade_history);
}