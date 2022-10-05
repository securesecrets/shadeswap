use crate::{
    contract::INSTANTIATE_REPLY_ID,
    state::{
        amm_pair_keys_r, amm_pair_keys_w, amm_pairs_r, amm_pairs_w, config_r, config_w,
        ephemeral_storage_w, prng_seed_r, total_amm_pairs_r, total_amm_pairs_w, NextPairKey,
        PAGINATION_LIMIT,
    },
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Querier,
    Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg,
};
use shadeswap_shared::{
    amm_pair::{generate_pair_key, AMMPair, AMMSettings},
    core::{admin_r, Callback, ContractInstantiationInfo, ContractLink, TokenPair, ViewingKey},
    msg::{
        amm_pair::InitMsg as AMMPairInitMsg,
        factory::{ExecuteMsg, InitMsg, QueryMsg, QueryResponse},
        router::ExecuteMsg as RouterExecuteMsg,
    },
    stake_contract::StakingContractInit,
    Pagination,
};

pub fn register_amm_pair(
    storage: &mut dyn Storage,
    _env: Env,
    pair: AMMPair,
) -> StdResult<Response> {
    add_amm_pairs(storage, vec![pair])
}

pub fn add_amm_pairs(storage: &mut dyn Storage, amm_pairs: Vec<AMMPair>) -> StdResult<Response> {
    for amm_pair in amm_pairs {
        let new_key = generate_pair_key(&amm_pair.pair);
        let existing_pair = amm_pair_keys_r(storage).may_load(&new_key)?;

        match existing_pair {
            Some(_) => {
                return Err(StdError::generic_err(format!(
                    "AMM Pair ({}) already exists",
                    amm_pair.pair
                )))
            }
            None => {
                let total_count_singleton = total_amm_pairs_r(storage);
                let current_count = total_count_singleton.may_load()?;
                let next_count = current_count.unwrap_or(0);
                amm_pair_keys_w(storage).save(&new_key, &amm_pair.address)?;
                amm_pairs_w(storage).save(&next_count.to_string().as_bytes(), &amm_pair)?;
                total_amm_pairs_w(storage).save(&(next_count + 1))?;
            }
        }
    }

    Ok(Response::new().add_attribute("action", "register_amm_pairs"))
}

pub fn list_pairs(deps: Deps, pagination: Pagination) -> StdResult<Binary> {
    let amm_pairs = load_amm_pairs(deps, pagination)?;

    to_binary(&QueryResponse::ListAMMPairs { amm_pairs })
}

pub fn query_amm_settings(deps: Deps) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;

    Ok(to_binary(&QueryResponse::GetAMMSettings {
        settings: config.amm_settings,
    })?)
}

pub fn query_amm_pair_address(deps: &Deps, pair: TokenPair) -> StdResult<Binary> {
    let address = amm_pair_keys_r(deps.storage).load(&generate_pair_key(&pair))?;
    to_binary(&QueryResponse::GetAMMPairAddress {
        address: address.to_string(),
    })
}

pub fn set_config(
    pair_contract: Option<ContractInstantiationInfo>,
    lp_token_contract: Option<ContractInstantiationInfo>,
    amm_settings: Option<AMMSettings>,
    storage: &mut dyn Storage,
    api_key: Option<String>,
) -> StdResult<Response> {
    let mut config = config_r(storage).load()?;
    if let Some(new_value) = pair_contract {
        config.pair_contract = new_value;
    }

    if let Some(new_value) = lp_token_contract {
        config.lp_token_contract = new_value;
    }

    if let Some(new_value) = amm_settings {
        config.amm_settings = new_value;
    }
    if let Some(new_value) = api_key {
        config.api_key = ViewingKey(new_value);
    }

    config_w(storage).save(&config)?;

    Ok(Response::default())
}

pub fn create_pair(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    pair: TokenPair,
    sender: Addr,
    entropy: Binary,
    staking_contract: Option<StakingContractInit>,
    router_contract: Option<ContractLink>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    let admin = admin_r(deps.storage).load()?;
    let signature = create_signature(&env, info)?;
    ephemeral_storage_w(deps.storage).save(&NextPairKey {
        pair: pair.clone(),
        is_verified: admin == sender,
        key: signature.clone(),
    })?;

    let mut messages = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: config.pair_contract.id,
        label: format!(
            "{}-{}-pair-{}-{}",
            pair.0, pair.1, env.contract.address, config.pair_contract.id
        ),
        msg: to_binary(&AMMPairInitMsg {
            pair: pair.clone(),
            lp_token_contract: config.lp_token_contract.clone(),
            factory_info: ContractLink {
                code_hash: env.contract.code_hash.clone(),
                address: env.contract.address.clone(),
            },
            entropy,
            prng_seed: prng_seed_r(deps.storage).load()?,
            admin: Some(admin_r(deps.storage).load()?),
            staking_contract: staking_contract,
            custom_fee: None,
            callback: Some(Callback {
                msg: to_binary(&ExecuteMsg::RegisterAMMPair {
                    pair: pair.clone(),
                    signature: signature,
                })?,
                contract: ContractLink {
                    address: env.contract.address,
                    code_hash: env.contract.code_hash,
                },
            }),
        })?,
        code_hash: config.pair_contract.code_hash,
        funds: vec![],
    }));

    if let Some(r) = router_contract {
        for p in pair.into_iter() {
            match p {
                shadeswap_shared::core::TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: r.address.to_string(),
                        code_hash: r.code_hash.to_string(),
                        msg: to_binary(&RouterExecuteMsg::RegisterSNIP20Token {
                            token_addr: contract_addr.clone(),
                            token_code_hash: token_code_hash.clone(),
                        })?,
                        funds: vec![],
                    }));
                }
                _ => (),
            }
        }
    }

    Ok(Response::new().add_messages(messages))
}

pub(crate) fn load_amm_pairs(deps: Deps, pagination: Pagination) -> StdResult<Vec<AMMPair>> {
    let count = total_amm_pairs_r(deps.storage).may_load()?;

    match count {
        Some(c) => {
            if pagination.start >= c {
                return Ok(vec![]);
            }

            let limit = pagination.limit.min(PAGINATION_LIMIT);
            let end = (pagination.start + limit as u64).min(c);

            let mut result = Vec::with_capacity((end - pagination.start) as usize);

            for i in pagination.start..end {
                let exchange: AMMPair = amm_pairs_r(deps.storage).load(i.to_string().as_bytes())?;

                result.push(exchange);
            }

            Ok(result)
        }
        None => Ok(vec![]),
    }
}

pub(crate) fn create_signature(env: &Env, info: &MessageInfo) -> StdResult<Binary> {
    to_binary(
        &[
            info.sender.as_bytes(),
            &env.block.height.to_be_bytes(),
            &env.block.time.seconds().to_be_bytes(),
        ]
        .concat(),
    )
}
