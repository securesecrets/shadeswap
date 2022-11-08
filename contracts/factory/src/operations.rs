use crate::{
    state::{
        amm_pair_keys_r, amm_pair_keys_w, amm_pairs_r, amm_pairs_w, config_r, config_w,
        ephemeral_storage_w, prng_seed_r, total_amm_pairs_r, total_amm_pairs_w, NextPairKey,
        PAGINATION_LIMIT,
    },
};
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Storage, WasmMsg,
};
use shadeswap_shared::{
    amm_pair::{generate_pair_key, AMMPair, AMMSettings},
    core::{Callback, ContractInstantiationInfo, TokenPair, ViewingKey},
    msg::{
        amm_pair::InitMsg as AMMPairInitMsg,
        factory::{ExecuteMsg, QueryResponse},
        router::ExecuteMsg as RouterExecuteMsg,
    staking::StakingContractInit,
    },
    Pagination, Contract,
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
    admin_auth: Option<Contract>,
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
    if let Some(new_value) = admin_auth {
        config.admin_auth = new_value;
    }

    config_w(storage).save(&config)?;

    Ok(Response::default())
}

pub fn create_pair(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    pair: TokenPair,
    entropy: Binary,
    staking_contract: Option<StakingContractInit>
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    let signature = create_signature(&env, info)?;
    ephemeral_storage_w(deps.storage).save(&NextPairKey {
        pair: pair.clone(),
        is_verified: true,
        key: signature.clone(),
    })?;

    let mut messages: Vec<CosmosMsg> = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: config.pair_contract.id,
        label: format!(
            "{}-{}-pair-{}-{}",
            pair.0, pair.1, env.contract.address, config.pair_contract.id
        ),
        msg: to_binary(&AMMPairInitMsg {
            pair: pair.clone(),
            lp_token_contract: config.lp_token_contract.clone(),
            factory_info: Contract {
                code_hash: env.contract.code_hash.clone(),
                address: env.contract.address.clone(),
            },
            entropy: entropy,
            prng_seed: prng_seed_r(deps.storage).load()?,
            admin_auth: config.admin_auth,
            staking_contract: staking_contract,
            custom_fee: None,
            callback: Some(Callback {
                msg: to_binary(&ExecuteMsg::RegisterAMMPair {
                    pair: pair.clone(),
                    signature: signature,
                })?,
                contract: Contract {
                    address: env.contract.address,
                    code_hash: env.contract.code_hash,
                },
            }),
        })?,
        code_hash: config.pair_contract.code_hash,
        funds: vec![],
    }));
    
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
