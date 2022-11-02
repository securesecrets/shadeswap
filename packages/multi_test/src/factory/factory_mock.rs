pub mod factory_mock {
    use crate::util_addr::util_addr::OWNER;
    use cosmwasm_std::{
        entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env,
        MessageInfo, Response, StdResult,
    };
    use cosmwasm_storage::{singleton, singleton_read};
    use schemars::JsonSchema;
    use shadeswap_shared::utils::asset::Contract;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{
        amm_pair::AMMSettings,
        core::{ContractInstantiationInfo, Fee},
        factory::{ExecuteMsg, QueryMsg, QueryResponse},
        utils::{pad_query_result, pad_response_result},
    };
    use shadeswap_shared::Contract as sContract;

    pub static CONFIG: &[u8] = b"config";
    pub const BLOCK_SIZE: usize = 256;   

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub admin_auth: Contract
    }

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        msg: InitMsg,
    ) -> StdResult<Response> {
        singleton(deps.storage, CONFIG).save(&msg.admin_auth)?;
        Ok(Response::new())
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::ListAMMPairs { pagination } => to_binary(""),
                QueryMsg::GetAMMPairAddress { pair } => to_binary(""),
                QueryMsg::GetConfig => {
                    println!("getconfig factory");
                    let admin_auth: Contract = singleton_read(deps.storage, CONFIG).load()?;
                    to_binary(&QueryResponse::GetConfig {
                        pair_contract: ContractInstantiationInfo {
                            code_hash: "".to_string(),
                            id: 0u64,
                        },
                        amm_settings: AMMSettings {
                            lp_fee: Fee::new(3, 100),
                            shade_dao_fee: Fee::new(3, 100),
                            shade_dao_address: sContract {
                                address: Addr::unchecked(OWNER),
                                code_hash: "".to_string(),
                            },
                        },
                        lp_token_contract: ContractInstantiationInfo {
                            code_hash: "".to_string(),
                            id: 0u64,
                        },
                        authenticator: None,
                        admin_auth: admin_auth.clone(),
                    })
                },
                QueryMsg::AuthorizeApiKey { api_key } => to_binary(""),
            },
            BLOCK_SIZE,
        )
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::SetConfig {
                    pair_contract,
                    lp_token_contract,
                    amm_settings,
                    api_key,
                    admin_auth,
                } => Ok(Response::new()),
                ExecuteMsg::CreateAMMPair {
                    pair,
                    entropy,
                    staking_contract,
                    router_contract,
                } => Ok(Response::new()),
                ExecuteMsg::AddAMMPairs { amm_pairs } => Ok(Response::new()),
                ExecuteMsg::RegisterAMMPair { pair, signature } => Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }
}
