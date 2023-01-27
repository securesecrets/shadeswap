use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, Env, QuerierWrapper, QueryRequest, StdError, StdResult, Uint128,
    WasmQuery,
};
use shadeswap_shared::{
    amm_pair::{AMMSettings, QueryMsgResponse, TradeHistory, FeeInfo},
    core::{Fee, TokenAmount, TokenPairAmount, TokenType},
    factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse},
    snip20::helpers::token_info,
    Contract, Pagination,
};

use crate::{
    operations::{calculate_lp_tokens, calculate_swap_result, lp_virtual_swap},
    state::{config_r, trade_count_r, trade_history_r, PAGINATION_LIMIT},
};

pub struct FactoryConfig {
    pub amm_settings: AMMSettings,
    pub authenticator: Option<Contract>,
    pub admin_auth: Contract,
}

pub fn factory_config(deps: Deps, factory: &Contract) -> StdResult<FactoryConfig> {
    let result: FactoryQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: factory.address.to_string(),
            msg: to_binary(&FactoryQueryMsg::GetConfig {})?,
            code_hash: factory.code_hash.to_string(),
        }))?;

    match result {
        FactoryQueryResponse::GetConfig {
            pair_contract: _,
            amm_settings,
            lp_token_contract: _,
            authenticator,
            admin_auth,
        } => Ok(FactoryConfig {
            amm_settings,
            authenticator,
            admin_auth,
        }),
        _ => Err(StdError::generic_err(
            "An error occurred while trying to retrieve factory settings.",
        )),
    }
}

pub fn swap_simulation(deps: Deps, env: Env, offer: TokenAmount, exclude_fee: Option<bool>) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;

    let fee_info = fee_info(deps)?;

    let swap_result = calculate_swap_result(
        deps,
        &env,
        fee_info.lp_fee,
        fee_info.shade_dao_fee,
        &config,
        &offer,
        exclude_fee,
        false
    )?;
    let simulation_result = QueryMsgResponse::SwapSimulation {
        total_fee_amount: swap_result.total_fee_amount,
        lp_fee_amount: swap_result.lp_fee_amount,
        shade_dao_fee_amount: swap_result.shade_dao_fee_amount,
        result: swap_result.result,
        price: swap_result.price,
    };
    to_binary(&simulation_result)
}

pub fn fee_info(deps: Deps) -> StdResult<FeeInfo> {
    let shade_dao_address: Addr;
    let lp_fee: Fee;
    let shade_dao_fee: Fee;

    let config = config_r(deps.storage).load()?;

    let amm_settings: Option<AMMSettings> = if let Some(factory_contract) = &config.factory_contract {
        Some(factory_config(deps, factory_contract)?.amm_settings)
    } else {
        None
    };

    //set dao address
    shade_dao_address = if let Some(amm_settings) = &amm_settings {
        amm_settings.shade_dao_address.address.clone()
    } else {
        // if no address is given then this address is used
        Addr::unchecked("")
    };

    //set fees
    match &config.custom_fee {
        Some(custom_fee) => {
            lp_fee = custom_fee.lp_fee;
            shade_dao_fee = custom_fee.shade_dao_fee;
        },
        None => {
            match &amm_settings {
                Some(amm_settings) => {
                    lp_fee = amm_settings.lp_fee;
                    shade_dao_fee = amm_settings.shade_dao_fee;
                }
                None => {
                    return Err(StdError::generic_err(
                        "Custom fee must be set if factory is not given.",
                    ))
                }
            }
        }
    }

    Ok(FeeInfo {
        shade_dao_address,
        lp_fee,
        shade_dao_fee,
    })
}

pub fn shade_dao_info(deps: Deps) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    let fee_info = fee_info(deps)?;
    let shade_dao_info = QueryMsgResponse::GetShadeDaoInfo {
        shade_dao_address: fee_info.shade_dao_address.to_string(),
        shade_dao_fee: fee_info.shade_dao_fee,
        admin_auth: config.admin_auth,
        lp_fee: fee_info.lp_fee,
    };
    to_binary(&shade_dao_info)
}

pub fn estimated_liquidity(deps: Deps, env: Env, deposit: &TokenPairAmount, sender: Addr, execute_sslp_virtual_swap: Option<bool>) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;

    if config.pair != deposit.pair {
        return Err(StdError::generic_err(
            "The provided tokens dont match those managed by the contract.",
        ));
    }

    let pool_balances = deposit.pair.query_balances(
        deps,
        env.contract.address.to_string(),
        config.viewing_key.0.clone(),
    )?;

    let pair_contract_pool_liquidity = total_supply(deps, &config.lp_token)?;

    let fee_info = fee_info(deps)?;

    let new_deposit =
        if let Some(false) = execute_sslp_virtual_swap {
            deposit.clone()
        } else {
            lp_virtual_swap(
                deps,
                &env,
                sender,
                fee_info.lp_fee,
                fee_info.shade_dao_fee,
                fee_info.shade_dao_address,
                &config,
                &deposit,
                pair_contract_pool_liquidity,
                pool_balances,
                None,
            )?
        };

    let lp_tokens = calculate_lp_tokens(&new_deposit, pool_balances, pair_contract_pool_liquidity)?;
    let response_msg = QueryMsgResponse::GetEstimatedLiquidity {
        lp_token: lp_tokens,
        total_lp_token: pair_contract_pool_liquidity,
    };
    to_binary(&response_msg)
}

pub fn token_symbol(querier: QuerierWrapper, token: &TokenType) -> StdResult<String> {
    match token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            return Ok(token_info(
                &querier,
                &Contract {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
            )?
            .symbol);
        }
        TokenType::NativeToken { denom: d } => {
            if d == "uscrt" {
                Ok("SCRT".to_string())
            } else {
                Ok(d.to_string())
            }
        }
    }
}

pub fn factory_authorize_api_key(
    deps: Deps,
    factory: &Contract,
    api_key: String,
) -> StdResult<bool> {
    let result: FactoryQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: factory.address.to_string(),
            msg: to_binary(&FactoryQueryMsg::AuthorizeApiKey { api_key: api_key })?,
            code_hash: factory.code_hash.to_string(),
        }))?;

    match result {
        FactoryQueryResponse::AuthorizeApiKey { authorized } => {
            if !authorized {
                return Err(StdError::generic_err(
                    "Authorization failed, key is incorrect.",
                ));
            }
            Ok(authorized)
        }
        _ => Err(StdError::generic_err(
            "Authorization failed, could not query factory successfully.",
        )),
    }
}

pub fn total_supply(deps: Deps, lp_token_info: &Contract) -> StdResult<Uint128> {
    let result = token_info(
        &deps.querier,
        &Contract {
            address: lp_token_info.address.clone(),
            code_hash: lp_token_info.code_hash.clone(),
        },
    )?;

    if let Some(ts) = result.total_supply {
        Ok(ts)
    } else {
        return Err(StdError::generic_err("LP token has no available supply."));
    }
}

pub fn trade_history(deps: Deps, count: u64) -> StdResult<TradeHistory> {
    let trade_history: TradeHistory =
        trade_history_r(deps.storage).load(count.to_string().as_bytes())?;
    Ok(trade_history)
}

pub fn trade_history_page(deps: Deps, pagination: Pagination) -> StdResult<Vec<TradeHistory>> {
    let count = trade_count_r(deps.storage).may_load()?.unwrap_or(0u64);

    if pagination.start >= count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    for i in pagination.start..end {
        let temp_index = i + 1;
        let trade_history: TradeHistory = trade_history(deps, temp_index)?;
        result.push(trade_history);
    }

    Ok(result)
}
