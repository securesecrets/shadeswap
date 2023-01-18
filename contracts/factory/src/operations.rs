use crate::{
    contract::INSTANTIATE_REPLY_ID,
    state::{
        amm_pair_keys_r, amm_pair_keys_w, amm_pairs_w, config_r, config_w, ephemeral_storage_w,
        prng_seed_r, total_amm_pairs_r, total_amm_pairs_w, NextPairKey, amm_pairs_r,
    },
};
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, Storage, SubMsg,
    WasmMsg,
};
use shadeswap_shared::{
    amm_pair::{generate_pair_key, AMMPair, AMMSettings},
    core::{ContractInstantiationInfo, TokenPair, ViewingKey},
    msg::{amm_pair::InitMsg as AMMPairInitMsg, staking::StakingContractInit},
    Contract,
};

pub fn register_amm_pair(storage: &mut dyn Storage, pair: AMMPair) -> StdResult<Response> {
    add_amm_pairs(storage, vec![pair])
}

pub fn add_amm_pairs(storage: &mut dyn Storage, amm_pairs: Vec<AMMPair>) -> StdResult<Response> {
    for amm_pair in amm_pairs {
        let new_key = generate_pair_key(&amm_pair.pair);
        let existing_pair = amm_pair_keys_r(storage).may_load(&new_key)?;
        let total_count_singleton: u64 = total_amm_pairs_r(storage).may_load()?.unwrap_or(0u64);

        match existing_pair {
            Some(e_p) => {
                amm_pair_keys_w(storage).save(&new_key, &amm_pair.address)?;
                for i in 0..total_count_singleton {
                    let existing_pair = amm_pairs_r(storage).load(&i.to_string().as_bytes())?;
                    if existing_pair.pair == amm_pair.pair {
                        amm_pairs_w(storage).save(&i.to_string().as_bytes(), &amm_pair)?;
                        break;
                    }
                }
            }
            None => {
                let next_count = total_count_singleton;
                amm_pair_keys_w(storage).save(&new_key, &amm_pair.address)?;
                amm_pairs_w(storage).save(&next_count.to_string().as_bytes(), &amm_pair)?;
                total_amm_pairs_w(storage).save(&(next_count + 1))?;
            }
        }
    }

    Ok(Response::new().add_attribute("action", "register_amm_pairs"))
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
    pair: TokenPair,
    entropy: Binary,
    staking_contract: Option<StakingContractInit>,
    lp_token_decimals: u8,
    amm_pair_custom_label: Option<String>,
    lp_token_custom_label: Option<String>
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    ephemeral_storage_w(deps.storage).save(&NextPairKey {
        pair: pair.clone(),
        code_hash: config.pair_contract.code_hash.to_string(),
    })?;

    let mut messages = vec![];
    messages.push(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pair_contract.id,
            label: amm_pair_custom_label.unwrap_or(format!(
                "{}-{}-pair-{}-{}",
                pair.0, pair.1, env.contract.address, config.pair_contract.id
            )),
            msg: to_binary(&AMMPairInitMsg {
                pair: pair.clone(),
                lp_token_contract: config.lp_token_contract.clone(),
                factory_info: Some(Contract {
                    code_hash: env.contract.code_hash.clone(),
                    address: env.contract.address.clone(),
                }),
                entropy: entropy,
                prng_seed: prng_seed_r(deps.storage).load()?,
                admin_auth: config.admin_auth,
                staking_contract: staking_contract,
                custom_fee: None,
                arbitrage_contract: None,
                lp_token_decimals: lp_token_decimals,
                lp_token_custom_label,
            })?,
            code_hash: config.pair_contract.code_hash,
            funds: vec![],
        }),
        INSTANTIATE_REPLY_ID,
    ));

    Ok(Response::new().add_submessages(messages))
}
