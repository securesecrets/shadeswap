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
use std::any::type_name;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tradehistory::TradeHistory;

pub static CONFIG_KEY: &[u8] = b"config";
const TRADE_COUNT : &[u8] = b"trade_count";
const TRADE_HISTORY: &[u8] = b"trade_history_";
const WHITELIST: &[u8] = b"whitelist";

pub const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct Config<A: Clone> {
    pub factory_info:  ContractLink<A>,
    pub lp_token_info: ContractLink<A>,
    pub pair:      TokenPair<A>,
    pub contract_addr: A,
    pub viewing_key: ViewingKey,
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
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
            factory_info:  self.factory_info.humanize(api)?,
            lp_token_info: self.lp_token_info.humanize(api)?,
            pair:          self.pair.humanize(api)?,
            contract_addr: self.contract_addr.humanize(api)?,
            viewing_key:   self.viewing_key.clone(),
        })
    }
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
        pub price: Uint128,
    }
    
    #[derive(Serialize, Deserialize,  PartialEq, Debug)]
    pub struct SwapResult {
        pub return_amount: Uint128,
        pub spread_amount: Uint128,
    }
    
}

pub mod amm_pair_storage{
    use super::*;
    use tradehistory::{DirectionType};

    pub fn store_config <S: Storage, A: Api, Q: Querier>(
        deps:   &mut Extern<S, A, Q>,
        config: &Config<HumanAddr>
    ) -> StdResult<()> {
        save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
    }
    
    pub fn load_config<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>
    ) -> StdResult<Config<HumanAddr>> {
        let result: Config<CanonicalAddr> = load(&deps.storage, CONFIG_KEY)?.ok_or(
            StdError::generic_err("Config doesn't exist in storage.")
        )?;
        result.humanize(&deps.api)
    }

    pub fn load_trade_counter(storage: &impl Storage) -> StdResult<u64> {
        Ok(load(storage, TRADE_COUNT)?.unwrap_or(0))
    }

    fn ser_bin_data<T: Serialize>(obj: &T) -> StdResult<Vec<u8>> {
        bincode2::serialize(&obj).map_err(|e| StdError::serialize_err(type_name::<T>(), e))
    }
    
    fn deser_bin_data<T: DeserializeOwned>(data: &[u8]) -> StdResult<T> {
        bincode2::deserialize::<T>(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))
    }
    
    // WHITELIST
    pub fn add_whitelist_address(storage: &mut impl Storage, address: HumanAddr) -> StdResult<()> {
        let mut unwrap_data = load_whitelist_address(storage)?;
        unwrap_data.push(address);
        let array = ser_bin_data(&unwrap_data)?;
        save(storage, WHITELIST, &unwrap_data)
    }

    pub fn load_whitelist_address(storage: &impl Storage) -> StdResult<Vec<HumanAddr>> {
        let raw_data = load(storage, WHITELIST)?.unwrap_or(Vec::new());        
        let unwrap_date = deser_bin_data(&raw_data)?;
        Ok(unwrap_date)
    }

    
    pub fn remove_whitelist_address(storage: &mut impl Storage, address_to_remove: Vec<HumanAddr>) -> StdResult<()> {
        let mut addresses = load_whitelist_address(storage)?;
        for address in address_to_remove {
            addresses.retain(|x| x != &address);
        }      
        save(storage, WHITELIST,&addresses)
    }

    pub fn is_address_in_whitelist(storage: &impl Storage, address: HumanAddr) -> StdResult<bool>{
        let mut result = false;
        let addrs = load_whitelist_address(storage)?;
        for addr in addrs {
            if addr == address
            {
                result = true;
            }
        }
        Ok(result)
    }

    // TRADE HISTORY
    fn store_trade_counter(storage: &mut impl Storage) -> StdResult<()> {
        let current_index = load_trade_counter(storage)?;
        let count : u64 = u64::from(current_index.clone());
        let str_count = count + 1;
        save(storage, TRADE_COUNT, &str_count)
    }
    
    pub fn load_trade_history(storage: &impl Storage, count: u64) -> StdResult<TradeHistory> {
        let temp_count = count;
        let count_to_string = temp_count.to_string();
        let count_bytes: &[u8] = count_to_string.as_bytes();
        let new_store_at = [TRADE_HISTORY.to_owned(), count_bytes.to_vec()].concat();  
        Ok(load(storage, &new_store_at)?.unwrap_or(TradeHistory{amount :Uint128::zero(), price: Uint128::zero(), direction: DirectionType::Unknown, timestamp: 0 }))
    }
    
    pub fn store_trade_history(storage: &mut impl Storage, trade_history: TradeHistory) -> StdResult<()> {
        let current_index: u64 = load_trade_counter(storage)?;
        let count = current_index.clone() + 1;
        let count_to_string  = count.to_string();
        let count_bytes: &[u8] = count_to_string .as_bytes();         
        let new_store_at = [TRADE_HISTORY.to_owned(), count_bytes.to_vec()].concat();   
        store_trade_counter(storage)?;
        save(storage, &new_store_at, &trade_history)
    }   
}