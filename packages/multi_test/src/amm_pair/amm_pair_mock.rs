pub mod amm_pair_mock {
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary, CosmosMsg, BankMsg, Coin, Addr, Uint128, ContractInfo, StdError, SubMsgResult, Reply, Storage, SubMsg, WasmMsg, from_binary};
    use cosmwasm_storage::{singleton, singleton_read, Singleton, ReadonlySingleton};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{utils::{pad_query_result, pad_response_result}, 
    msg::amm_pair::{ExecuteMsg, QueryMsg, QueryMsgResponse, SwapResult, InitMsg}, core::{TokenType, TokenPair, ContractLink, CustomFee, ViewingKey, admin_r, create_viewing_key}, staking::StakingContractInit, snip20::helpers::register_receive};
    use crate::{util_addr::util_addr::OWNER, help_lib::integration_help_lib::get_contract_link_from_token_type};    
    pub const BLOCK_SIZE: usize = 256;
    use crate::staking::staking_mock::staking_mock::InitMsg as StakingInitMsg;
    use shadeswap_shared::Contract;
    pub const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 1u64;
    pub const INSTANTIATE_STAKING_CONTRACT_REPLY_ID: u64 = 2u64;
    
    pub static CONFIG: &[u8] = b"config";
    pub static TOKEN_0: &[u8] = b"token_0";
    pub static TOKEN_1: &[u8] = b"token_1";
    pub static FACTORY: &[u8] = b"factory";

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InitMsg,
    ) -> StdResult<Response> { 
        let mut response = Response::new(); 
        let config = Config {
            factory_contract: msg.factory_info.clone(),
            lp_token: ContractLink {
                code_hash: msg.lp_token_contract.code_hash,
                address: Addr::unchecked(""),
            },
            pair: msg.pair,
            viewing_key: create_viewing_key(&env, &info, msg.prng_seed.clone(), msg.entropy.clone()),
            custom_fee: msg.custom_fee.clone(),
            staking_contract: None,
            staking_contract_init: msg.staking_contract,
            prng_seed: msg.prng_seed,
        };    
        singleton(deps.storage, CONFIG).save(&config)?;        
        match msg.callback {
            Some(c) => {
                response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: c.contract.address.to_string(),
                    code_hash: c.contract.code_hash,
                    msg: c.msg,
                    funds: vec![],
                }))
            }
            None => (),
        }        
        Ok(response.add_attribute("created_exchange_address", env.contract.address.to_string()))
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::GetConfig {  } => to_binary(""),
                QueryMsg::GetPairInfo {  } => {  
                    let config: Config = singleton_read(deps.storage, CONFIG).load()?;                  
                    let token_0 = get_contract_link_from_token_type(&config.pair.0);
                    let token_1: ContractLink = get_contract_link_from_token_type(&config.pair.0);                    
                    let response = QueryMsgResponse::GetPairInfo { 
                        liquidity_token: token_0.to_owned(), 
                        factory: config.factory_contract.to_owned(),
                        pair: TokenPair(
                            TokenType::CustomToken { contract_addr: token_0.address.to_owned(), token_code_hash: token_0.code_hash.to_owned() },
                            TokenType::CustomToken { contract_addr: token_1.address.to_owned(), token_code_hash: token_1.code_hash.to_owned() }
                        ), 
                        amount_0: Uint128::new(1000u128), 
                        amount_1: Uint128::new(1000u128),
                        total_liquidity: Uint128::new(1000000), 
                        contract_version: 1 
                    };
                    to_binary(&response)
                },
                QueryMsg::GetTradeHistory { api_key, pagination } => to_binary(""),
                QueryMsg::GetWhiteListAddress {  } => to_binary(""),
                QueryMsg::GetTradeCount {  } => to_binary(""),
                QueryMsg::GetAdmin {  } => to_binary(""),
                QueryMsg::GetStakingContract {  } => to_binary(""),
                QueryMsg::GetEstimatedPrice { offer, exclude_fee } => to_binary(""),
                QueryMsg::SwapSimulation { offer } => {
                    let response = QueryMsgResponse::SwapSimulation { 
                        total_fee_amount: Uint128::new(150u128), 
                        lp_fee_amount: Uint128::new(50u128),
                        shade_dao_fee_amount: Uint128::new(150u128),
                        result: SwapResult{ 
                            return_amount: offer.amount 
                        }, 
                        price: "1.2".to_string() 
                    };
                    return to_binary(&response)
                },
                QueryMsg::GetShadeDaoInfo {  } => to_binary(""),
                QueryMsg::GetEstimatedLiquidity { deposit, slippage } => to_binary(""),
            },
            BLOCK_SIZE,
        )
    }

    
    #[entry_point]
    pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::AddLiquidityToAMMContract { deposit, expected_return, staking } => Ok(Response::new()),
                ExecuteMsg::SwapTokens { offer, expected_return, to, router_link, callback_signature } =>Ok(Response::new()),
                ExecuteMsg::Receive { from, msg, amount } => Ok(Response::new()),
                ExecuteMsg::AddWhiteListAddress { address } => Ok(Response::new()),
                ExecuteMsg::RemoveWhitelistAddresses { addresses } =>Ok(Response::new()),
                ExecuteMsg::SetAdmin { admin } => Ok(Response::new()),
                ExecuteMsg::SetCustomPairFee { custom_fee } => Ok(Response::new()),
                ExecuteMsg::SetViewingKey { viewing_key } => Ok(Response::new()),
                ExecuteMsg::RecoverFunds { token, amount, to, msg } => Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }

    #[entry_point]
    pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
        pad_response_result(
            match (msg.id, msg.result) {
                (INSTANTIATE_LP_TOKEN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                    Some(x) => {   
                        let mut temp = String::from_utf8(x.to_vec())?;
                        temp = temp.replace("(", "");
                        temp = temp.replace("\n", "");
                        let address = &temp[..40];     
                        let contract_address = Addr::unchecked(address);                        
                        let config = config_r(deps.storage).load()?;
                        register_lp_token(
                            deps,
                            _env,
                            Contract {
                                address: contract_address,
                                code_hash: config.lp_token.code_hash,
                            },
                        );
                        Ok(Response::new())
                    }
                    None => Err(StdError::generic_err(format!("Unknown reply id"))),
                },
                (INSTANTIATE_STAKING_CONTRACT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                    Some(x) => {
                        let contract_address = String::from_utf8(x.to_vec())?;
                        println!("staking address {}", contract_address);
                        let config = config_r(deps.storage).load()?;
                        set_staking_contract(
                            deps.storage,
                            Some(ContractLink {
                                address: Addr::unchecked(contract_address),
                                code_hash: config
                                    .staking_contract_init
                                    .ok_or(StdError::generic_err(
                                        "Staking contract does not match.".to_string(),
                                    ))?
                                    .contract_info
                                    .code_hash,
                            }),
                        );
                        Ok(Response::new())
                    }
                    None => Err(StdError::generic_err(format!("Unknown reply id"))),
                },
                _ => Err(StdError::generic_err(format!("Unknown reply id"))),
            },
            BLOCK_SIZE
        )     
    }

    pub fn set_staking_contract(
        storage: &mut dyn Storage,
        staking_contract: Option<ContractLink>,
    ) -> StdResult<Response> {
        let mut config = config_w(storage).load()?;
       
        config.staking_contract = staking_contract;
    
        config_w(storage).save(&config)?;
    
        // send lp contractLink to staking contract
        Ok(Response::new().add_attribute("action", "set_staking_contract"))
    }

    pub fn register_lp_token(
        deps: DepsMut,
        env: Env,
        lp_token_address: Contract,
    ) -> StdResult<Response> {
        let mut config = config_r(deps.storage).load()?;
       
        config.lp_token = ContractLink {
            address: lp_token_address.address.clone(),
            code_hash: lp_token_address.code_hash.clone(),
        };
        // store config against Smart contract address
        config_w(deps.storage).save(&config)?;
    
        let mut response = Response::new().add_message(register_receive(
            env.contract.code_hash.clone(),
            None,
            &lp_token_address.clone(),
        )?);    
      
        match config.staking_contract_init {
            Some(c) => {
                println!("register staking ");
                println!("lp address {}", &lp_token_address.address.to_string());
                println!("ShadeSwap-Pair-Staking-Contract-{}", &env.contract.address.to_string());
                response = response.add_submessage(SubMsg::reply_on_success(
                    CosmosMsg::Wasm(WasmMsg::Instantiate {
                        code_id: c.contract_info.id,
                        label: format!("ShadeSwap-Pair-Staking-Contract-{}", &env.contract.address),
                        msg: to_binary(&StakingInitMsg { })?,
                        code_hash: c.contract_info.code_hash.clone(),
                        funds: vec![],
                    }),
                    INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
                ));
            }
            _ => {
                ();
            }
        }
    
        Ok(response)
    }

    
    pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
        singleton(storage, CONFIG)
    }

    pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
        singleton_read(storage, CONFIG)
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    pub struct Config {
        pub factory_contract: ContractLink,
        pub lp_token: ContractLink,
        pub staking_contract: Option<ContractLink>,
        pub pair: TokenPair,
        pub viewing_key: ViewingKey,
        pub custom_fee: Option<CustomFee>,
        pub staking_contract_init: Option<StakingContractInit>,
        pub prng_seed: Binary
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    pub enum DirectionType {
        Buy,
        Sell,
        Unknown,
    }
}