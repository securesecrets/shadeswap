use crate::state::get_address_for_pair;
use crate::state::load_amm_pairs;
use crate::state::load_prng_seed;
use crate::state::save_amm_pairs;
use crate::state::save_prng_seed;
use secret_toolkit::utils::HandleCallback;
use secret_toolkit::utils::InitCallback;
use shadeswap_shared::amm_pair::AMMPair;
use shadeswap_shared::msg::factory::HandleMsg;
use shadeswap_shared::msg::factory::InitMsg;
use shadeswap_shared::msg::factory::QueryMsg;
use shadeswap_shared::msg::factory::QueryResponse;
use shadeswap_shared::Pagination;
use shadeswap_shared::TokenPair;
use shadeswap_shared::{
    fadroma::{
        admin::{
            assert_admin, handle as admin_handle, load_admin, query as admin_query, save_admin,
            DefaultImpl as AdminImpl,
        },
        require_admin::require_admin,
        scrt::{
            log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
        },
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_migrate,
        scrt_migrate::get_status,
        scrt_storage::{load, remove, save},
        with_status,
    },
    msg::amm_pair::InitMsg as AMMPairInitMsg,
};

use crate::state::{config_read, config_write, Config};

pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    save_prng_seed(&mut deps.storage, &msg.prng_seed)?;
    config_write(deps, &Config::from_init_msg(msg));

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    return match msg {
        HandleMsg::CreateAMMPair { pair, entropy } => create_pair(deps, env, pair, entropy),
        HandleMsg::SetConfig { .. } => set_config(deps, env, msg),
        HandleMsg::AddAMMPairs { amm_pair } => add_amm_pairs(deps, env, amm_pair),
        HandleMsg::RegisterAMMPair { pair, signature } => {
            register_amm_pair(deps, env, pair, signature)
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
        QueryMsg::GetAMMSettings { } => query_amm_settings(deps)
    }
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
    let address = get_address_for_pair(deps, &pair)?;
    to_binary(&QueryResponse::GetAMMPairAddress { address })
}

pub fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    if let HandleMsg::SetConfig { pair_contract, lp_token_contract, amm_settings } = msg {
        let mut config = config_read(&deps)?;

        if let Some(new_value) = pair_contract {
            config.pair_contract = new_value;
        }

        if let Some(new_value) = lp_token_contract {
            config.lp_token_contract = new_value;
        }
        
        config_write(deps, &config)?;

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
        lp_token_contract
    } = config_read(deps)?;

    to_binary(&QueryResponse::GetConfig {
        pair_contract,
        amm_settings,
        lp_token_contract
    })
}

pub fn create_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair<HumanAddr>,
    entropy: Binary
) -> StdResult<HandleResponse> {
    let mut config = config_read(&deps)?;

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
                admin: Some(env.message.sender.clone())
            })?,
        })],
        log: vec![log("action", "create_exchange"), log("pair", pair)],
        data: None,
    })
}

fn ensure_correct_signature(storage: &mut impl Storage, signature: Binary) -> StdResult<()> {
    let stored_signature: Binary = load(storage, EPHEMERAL_STORAGE_KEY)?.unwrap_or_default();

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
