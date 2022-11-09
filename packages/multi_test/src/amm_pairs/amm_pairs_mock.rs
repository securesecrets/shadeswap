pub mod amm_pairs_mock {
    use crate::{
        help_lib::integration_help_lib::get_contract_link_from_token_type        
    };
    use cosmwasm_std::{
        entry_point, to_binary, Addr, Binary, CosmosMsg,
        Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, Storage, SubMsg,
        SubMsgResult, Uint128, WasmMsg, QueryRequest, WasmQuery,
    };
    use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{
        core::{
            create_viewing_key, CustomFee, TokenPair, TokenType, ViewingKey,
        },
        msg::amm_pair::{ExecuteMsg, InitMsg, QueryMsg, QueryMsgResponse, SwapResult},
        snip20::helpers::register_receive,
        staking::StakingContractInit,
        utils::{pad_query_result, pad_response_result}, amm_pair::AMMSettings,
    };
    use amm_pair::operations::register_lp_token;
    use amm_pair::state::config_r;
    use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse, QueryMsg as FactoryQueryMsg};
    pub const BLOCK_SIZE: usize = 256;
    //use crate::staking::staking_mock::staking_mock::InitMsg as StakingInitMsg;
    use shadeswap_shared::msg::staking::InitMsg as StakingInitMsg;
    use shadeswap_shared::Contract;
    pub const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 1u64;
    pub const INSTANTIATE_STAKING_CONTRACT_REPLY_ID: u64 = 2u64;

    pub static CONFIG: &[u8] = b"config";
    pub static TOKEN_0: &[u8] = b"token_0";
    pub static TOKEN_1: &[u8] = b"token_1";
    pub static FACTORY: &[u8] = b"factory";
    use amm_pair::operations::set_staking_contract;
    
    struct FactoryConfig {
        amm_settings: AMMSettings,
        authenticator: Option<Contract>,
        admin_auth: Contract
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    pub struct Config {
        pub factory_contract: Contract,
        pub lp_token: Contract,
        pub staking_contract: Option<Contract>,
        pub pair: TokenPair,
        pub viewing_key: ViewingKey,
        pub custom_fee: Option<CustomFee>,
        pub staking_contract_init: Option<StakingContractInit>,
        pub prng_seed: Binary,
        pub admin_auth: Contract
    }

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
            lp_token: Contract {
                code_hash: msg.lp_token_contract.code_hash,
                address: Addr::unchecked(""),
            },
            pair: msg.pair,
            viewing_key: create_viewing_key(
                &env,
                &info,
                msg.prng_seed.clone(),
                msg.entropy.clone(),
            ),
            custom_fee: msg.custom_fee.clone(),
            staking_contract: None,
            staking_contract_init: msg.staking_contract,
            prng_seed: msg.prng_seed,
            admin_auth: msg.admin_auth
        };
        singleton(deps.storage, CONFIG).save(&config)?;
        Ok(response.add_attribute("created_exchange_address", env.contract.address.to_string()))
    }

    #[entry_point]
    pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::GetConfig {} => to_binary(""),
                QueryMsg::GetPairInfo {} => {
                    let config: Config = singleton_read(deps.storage, CONFIG).load()?;
                    let token_0 = get_contract_link_from_token_type(&config.pair.0);
                    let token_1: Contract = get_contract_link_from_token_type(&config.pair.0);
                    let response = QueryMsgResponse::GetPairInfo {
                        liquidity_token: token_0.to_owned(),
                        factory: config.factory_contract.to_owned(),
                        pair: TokenPair(
                            TokenType::CustomToken {
                                contract_addr: token_0.address.to_owned(),
                                token_code_hash: token_0.code_hash.to_owned(),
                            },
                            TokenType::CustomToken {
                                contract_addr: token_1.address.to_owned(),
                                token_code_hash: token_1.code_hash.to_owned(),
                            },
                        ),
                        amount_0: Uint128::new(1000u128),
                        amount_1: Uint128::new(1000u128),
                        total_liquidity: Uint128::new(1000000),
                        contract_version: 1,
                    };
                    to_binary(&response)
                }
                QueryMsg::GetTradeHistory {
                    api_key: _,
                    pagination: _,
                } => to_binary(""),
                QueryMsg::GetWhiteListAddress {} => to_binary(""),
                QueryMsg::GetTradeCount {} => to_binary(""),
                QueryMsg::GetStakingContract {} => to_binary(""),
                QueryMsg::GetEstimatedPrice { offer: _, exclude_fee: _ } => to_binary(""),
                QueryMsg::SwapSimulation { offer } => {
                    let response = QueryMsgResponse::SwapSimulation {
                        total_fee_amount: Uint128::new(150u128),
                        lp_fee_amount: Uint128::new(50u128),
                        shade_dao_fee_amount: Uint128::new(150u128),
                        result: SwapResult {
                            return_amount: offer.amount,
                        },
                        price: "1.2".to_string(),
                    };
                    return to_binary(&response);
                }
                QueryMsg::GetShadeDaoInfo {} => to_binary(""),
                QueryMsg::GetEstimatedLiquidity { deposit: _ } => to_binary(""),
            },
            BLOCK_SIZE,
        )
    }

    #[entry_point]
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::AddLiquidityToAMMContract {
                    deposit: _,
                    expected_return: _,
                    staking: _,
                } => Ok(Response::new()),
                ExecuteMsg::SwapTokens {
                    offer: _,
                    expected_return: _,
                    to: _,
                } => Ok(Response::new()),
                ExecuteMsg::Receive {
                    from: _,
                    msg: _,
                    amount: _,
                } => Ok(Response::new()),
                ExecuteMsg::AddWhiteListAddress { address: _ } => Ok(Response::new()),
                ExecuteMsg::RemoveWhitelistAddresses { addresses: _ } => Ok(Response::new()),
                ExecuteMsg::SetCustomPairFee { custom_fee: _ } => Ok(Response::new()),
                ExecuteMsg::SetViewingKey { viewing_key: _ } => Ok(Response::new()),
                ExecuteMsg::RecoverFunds {
                    token: _,
                    amount: _,
                    to: _,
                    msg: _msg,
                } => Ok(Response::new()),
                ExecuteMsg::SetConfig { admin_auth: _ } => Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }

    #[entry_point]
    pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
        pad_response_result(
            match (msg.id, msg.result) {
                (INSTANTIATE_LP_TOKEN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                    Some(x) => {
                        let mut temp = String::from_utf8(x.to_vec())?;
                        temp = temp.replace("(", "");
                        temp = temp.replace("\n", "");
                        let address = &temp[..40];
                        let contract_address = Addr::unchecked(address);
                        println!("LP ADDRESS {}", address.to_string());
                        let config = config_r(deps.storage).load()?;
                        let mut response = register_lp_token(
                            deps,
                            &env,
                            Contract {
                                address: contract_address,
                                code_hash: config.lp_token.code_hash,
                            },
                        )?;                        
                        response.data = Some(env.contract.address.to_string().as_bytes().into());    
                        Ok(response)
                    }
                    None => Err(StdError::generic_err(format!("Unknown reply id"))),
                },
                (INSTANTIATE_STAKING_CONTRACT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                    Some(x) => {
                        let mut temp = String::from_utf8(x.to_vec())?;
                        temp = temp.replace("(", "");
                        temp = temp.replace("\n", "");
                        let address = &temp[..40];
                        let contract_address = Addr::unchecked(address);
                        println!("STAKING ADDRESS {}", address.to_string());
                        let config = config_r(deps.storage).load()?;
                        let mut response = set_staking_contract(
                            deps.storage,
                            Some(Contract {
                                address: contract_address,
                                code_hash: config
                                    .staking_contract_init
                                    .ok_or(StdError::generic_err(
                                        "Staking contract does not match.".to_string(),
                                    ))?
                                    .contract_info
                                    .code_hash,
                            }),
                        )?;                        
                        response.data = Some(env.contract.address.to_string().as_bytes().into());    
                        Ok(response)
                    }
                    None => Err(StdError::generic_err(format!("Unknown reply id"))),
                },
                _ => Err(StdError::generic_err(format!("Unknown reply id"))),
            },
            BLOCK_SIZE,
        )
    }

   
}
