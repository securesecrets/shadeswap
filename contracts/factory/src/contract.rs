use crate::state::{
    config_read, config_write, get_address_for_pair, load_amm_pairs, load_prng_seed,
    save_amm_pairs, save_prng_seed, Config,
};
use cosmwasm_std::{
    log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, WasmMsg,
};
use shadeswap_shared::{
    admin::{apply_admin_guard, load_admin, set_admin_guard, store_admin},
    amm_pair::AMMPair,
    fadroma::prelude::{Callback, ContractLink},
    msg::{
        amm_pair::InitMsg as AMMPairInitMsg,
        factory::{HandleMsg, InitMsg, QueryMsg, QueryResponse},
    },
    scrt_storage::{load, remove, save},
    stake_contract::StakingContractInit,
    Pagination, TokenPair,
};

pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    save_prng_seed(&mut deps.storage, &msg.prng_seed)?;
    config_write(deps, Config::from_init_msg(msg))?;
    store_admin(deps, &env.message.sender.clone())?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    return match msg {
        HandleMsg::CreateAMMPair {
            pair,
            entropy,
            staking_contract,
        } => create_pair(deps, env, pair, entropy, staking_contract),
        HandleMsg::SetConfig { .. } => set_config(deps, env, msg),
        HandleMsg::AddAMMPairs { amm_pairs } => add_amm_pairs(deps, env, amm_pairs),
        HandleMsg::RegisterAMMPair { pair, signature } => {
            register_amm_pair(deps, env, pair, signature)
        }
        HandleMsg::SetFactoryAdmin { admin } => set_admin_guard(deps, env, admin),
        HandleMsg::SetShadeDAOAddress { shade_dao_address } => {
            set_shade_dao_address(deps, env, shade_dao_address)
        }
    };
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => get_config(deps),
        QueryMsg::ListAMMPairs { pagination } => list_pairs(deps, pagination),
        QueryMsg::GetAMMPairAddress { pair } => query_amm_pair_address(deps, pair),
        QueryMsg::GetAMMSettings {} => query_amm_settings(deps),
        QueryMsg::GetAdmin {} => {
            let admin_address = load_admin(&deps.storage)?;
            to_binary(&QueryResponse::GetAdminAddress {
                address: admin_address,
            })
        }
    }
}

fn set_shade_dao_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    shade_dao_address: ContractLink<HumanAddr>,
) -> StdResult<HandleResponse> {
    apply_admin_guard(env.message.sender.clone(), &deps.storage)?;
    let mut config = config_read(deps)?;
    config.amm_settings.shade_dao_address = shade_dao_address.clone();
    config_write(deps, config)?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_amm_pair"),
            log("address", env.message.sender),
        ],
        data: None,
    })
}

fn register_amm_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair<HumanAddr>,
    signature: Binary,
) -> StdResult<HandleResponse> {
    ensure_correct_signature(&mut deps.storage, signature)?;
    let amm_pair = AMMPair {
        pair,
        address: env.message.sender.clone(),
    };
    save_amm_pairs(deps, vec![amm_pair])?;
    // create staking contract

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_amm_pair"),
            log("address", env.message.sender),
        ],
        data: None,
    })
}

pub fn add_amm_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amm_pairs: Vec<AMMPair<HumanAddr>>,
) -> StdResult<HandleResponse> {
    apply_admin_guard(env.message.sender.clone(), &deps.storage)?;
    save_amm_pairs(deps, amm_pairs)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

pub fn list_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Binary> {
    let amm_pairs = load_amm_pairs(deps, pagination)?;

    to_binary(&QueryResponse::ListAMMPairs { amm_pairs })
}

pub fn query_amm_settings<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config = config_read(deps)?;

    Ok(to_binary(&QueryResponse::GetAMMSettings {
        settings: config.amm_settings,
    })?)
}

pub fn query_amm_pair_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: TokenPair<HumanAddr>,
) -> StdResult<Binary> {
    let address = get_address_for_pair(deps, pair)?;
    to_binary(&QueryResponse::GetAMMPairAddress { address })
}

pub fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    if let HandleMsg::SetConfig {
        pair_contract,
        lp_token_contract,
        amm_settings,
    } = msg
    {
        let mut config = config_read(&deps)?;
        if let Some(new_value) = pair_contract {
            config.pair_contract = new_value;
        }

        if let Some(new_value) = lp_token_contract {
            config.lp_token_contract = new_value;
        }

        if let Some(new_value) = amm_settings {
            config.amm_settings = new_value;
        }

        config_write(deps, config)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        })
    } else {
        unreachable!()
    }
}

pub fn get_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let Config {
        pair_contract,
        amm_settings,
        lp_token_contract,
    } = config_read(deps)?;

    to_binary(&QueryResponse::GetConfig {
        pair_contract,
        amm_settings,
        lp_token_contract,
    })
}

pub fn create_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair<HumanAddr>,
    entropy: Binary,
    staking_contract: Option<StakingContractInit>,
) -> StdResult<HandleResponse> {
    let mut config = config_read(&deps)?;
    println!("create_pair caller {}", env.message.sender.clone());
    apply_admin_guard(env.message.sender.clone(), &deps.storage)?;
    //Used for verifying callback
    let signature = create_signature(&env)?;
    save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;
    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pair_contract.id,
            callback_code_hash: config.pair_contract.code_hash,
            send: vec![],
            label: format!(
                "{}-{}-pair-{}-{}",
                pair.0, pair.1, env.contract.address, config.pair_contract.id
            ),
            msg: to_binary(&AMMPairInitMsg {
                pair: pair.clone(),
                lp_token_contract: config.lp_token_contract.clone(),
                factory_info: ContractLink {
                    code_hash: env.contract_code_hash.clone(),
                    address: env.contract.address.clone(),
                },
                callback: Some(Callback {
                    contract: ContractLink {
                        address: env.contract.address,
                        code_hash: env.contract_code_hash,
                    },
                    msg: to_binary(&HandleMsg::RegisterAMMPair {
                        pair: pair.clone(),
                        signature,
                    })?,
                }),
                entropy,
                prng_seed: load_prng_seed(&deps.storage)?,
                admin: Some(env.message.sender.clone()),
                staking_contract: staking_contract,
                custom_fee: None,
            })?,
        })],
        log: vec![log("action", "create_exchange"), log("pair", pair)],
        data: None,
    })
}

fn ensure_correct_signature(storage: &mut impl Storage, signature: Binary) -> StdResult<()> {
    let stored_signature: Binary =
        load(storage, EPHEMERAL_STORAGE_KEY)?.ok_or_else(|| StdError::unauthorized())?;
    if stored_signature != signature {
        return Err(StdError::unauthorized());
    }

    remove(storage, EPHEMERAL_STORAGE_KEY);

    Ok(())
}

pub(crate) fn create_signature(env: &Env) -> StdResult<Binary> {
    to_binary(
        &[
            env.message.sender.0.as_bytes(),
            &env.block.height.to_be_bytes(),
            &env.block.time.to_be_bytes(),
        ]
        .concat(),
    )
}
