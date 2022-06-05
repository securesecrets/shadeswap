use shadeswap_shared::{
    fadroma::{
        scrt_link::{ContractLink},    
        scrt_addr::{Humanize, Canonize},
        scrt::{
            Api, CanonicalAddr, Extern, HumanAddr, Uint128,
            Querier, StdResult, Storage, StdError
        },
        scrt_storage::{load, save, ns_save, ns_load},
        scrt_vk::ViewingKey,
    },
    token_pair::TokenPair
};

use serde::{Deserialize, Serialize};

use shadeswap_shared::msg::amm_pair::{{ TradeHistory}};

pub const PAGINATION_LIMIT: u8 = 30;
pub static CONFIG_KEY: &[u8] = b"config";
pub static TRADE_COUNT: &[u8] = b"tradecount";
pub static TRADE_HISTORY: &[u8] = b"trade_history";
pub static WHITELIST: &[u8] = b"whitelist";
pub static ADMIN: &[u8] =b"amm_pair_admin";
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
    use shadeswap_shared::fadroma::Humanize;
    #[derive(Serialize, Deserialize,  PartialEq, Debug, Clone)]
    pub enum DirectionType{
        Buy,
        Sell,
        Unknown,
    }

    impl Humanize<String> for DirectionType {
        fn humanize(&self, api: &impl Api) -> StdResult<String> {
            match *self {
                DirectionType::Sell => Ok("Sell".to_string()),
                DirectionType::Buy => Ok("Buy".to_string()),
                DirectionType::Unknown => Ok("Unknown".to_string())
            }
        }
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
        pub price: Uint128
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
        let count = load(storage, TRADE_COUNT)?.unwrap_or(0);
        Ok(count)
    }
 
    pub fn store_trade_counter<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>, 
        count: u64
    ) -> StdResult<()> {      
        save(&mut deps.storage, TRADE_COUNT, &count)
    }   
    
    pub fn store_admin <S: Storage, A: Api, Q: Querier>(
        deps:   &mut Extern<S, A, Q>,
        admin: &HumanAddr
    ) -> StdResult<()> {
        save(&mut deps.storage, ADMIN, &admin)
    }

    pub fn load_admin(storage: &impl Storage) -> StdResult<HumanAddr> {
        let admin = load(storage, ADMIN)?.unwrap_or(HumanAddr("".to_string()));
        Ok(admin)
    }

    // WHITELIST
    pub fn add_whitelist_address(storage: &mut impl Storage, address: HumanAddr) -> StdResult<()> {
        let mut unwrap_data = load_whitelist_address(storage)?;
        unwrap_data.push(address);    
        save(storage, WHITELIST, &unwrap_data)
    }

    pub fn load_whitelist_address(storage: &impl Storage) -> StdResult<Vec<HumanAddr>> {
        let raw_data = load(storage, WHITELIST)?.unwrap_or(Vec::new());  
        Ok(raw_data)
    }

    
    pub fn remove_whitelist_address(storage: &mut impl Storage, address_to_remove: Vec<HumanAddr>) -> StdResult<()> {
        let mut addresses = load_whitelist_address(storage)?;
        for address in address_to_remove {
            addresses.retain(|x| x != &address);
        }
        save(storage, WHITELIST,&addresses)
    }

    pub fn is_address_in_whitelist(storage: &impl Storage, address: HumanAddr) -> StdResult<bool>{
        let addrs = load_whitelist_address(storage)?;
        if addrs.contains(&address) {
           return Ok(true)
        } else {
            return Ok(false)
        }      
    }

    pub fn load_trade_history<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        count: u64) -> StdResult<TradeHistory> {
        let trade_history: TradeHistory =
        ns_load(&deps.storage, TRADE_HISTORY, count.to_string().as_bytes())?
            .ok_or_else(|| StdError::generic_err("Trade History doesn't exist in storage."))?;
       Ok(trade_history)
    }
    
    pub fn store_trade_history<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>, 
        trade_history: &TradeHistory
    ) -> StdResult<()> {       
        let count = load_trade_counter(&deps.storage)?;                            
        let update_count = count + 1; 
        store_trade_counter(deps, update_count)?;
        ns_save(&mut deps.storage, TRADE_HISTORY, update_count.to_string().as_bytes(), &trade_history)
    }   
}