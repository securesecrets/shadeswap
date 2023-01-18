pub mod factory_mock {
    

    use crate::util_addr::util_addr::OWNER;
    use cosmwasm_std::{
        entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env,
        MessageInfo, Response, StdResult, StdError, SubMsgResult, Reply,
    };
    use factory::state::ephemeral_storage_r;
    use factory::operations::register_amm_pair;
    use cosmwasm_storage::{singleton, singleton_read};
    use schemars::{JsonSchema};
    use shadeswap_shared::{utils::asset::Contract, amm_pair::AMMPair};
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{
        amm_pair::AMMSettings,
        core::{ContractInstantiationInfo, Fee},
        factory::{ExecuteMsg, QueryMsg, QueryResponse},
        utils::{pad_query_result, pad_response_result},
    };
    use factory::state::ephemeral_storage_w;
    use shadeswap_shared::Contract as sContract;
    pub const INSTANTIATE_REPLY_ID: u64 = 1u64;
    
    pub static CONFIG: &[u8] = b"config";
    pub const BLOCK_SIZE: usize = 256;   

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub admin_auth: Contract
    }

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InitMsg,
    ) -> StdResult<Response> {
        singleton(deps.storage, CONFIG).save(&msg.admin_auth)?;
        Ok(Response::new())
    }

    #[entry_point]
    pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::ListAMMPairs { pagination: _ } => to_binary(""),
                QueryMsg::GetAMMPairAddress { pair: _ } => to_binary(""),
                QueryMsg::GetConfig {} => {
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
                QueryMsg::AuthorizeApiKey { api_key: _ } => to_binary(""),
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
                ExecuteMsg::SetConfig {
                    pair_contract: _,
                    lp_token_contract: _,
                    amm_settings: _,
                    api_key: _,
                    admin_auth: _,
                } => Ok(Response::new()),
                ExecuteMsg::CreateAMMPair {pair:_,entropy:_,staking_contract:_,lp_token_decimals:_u8, lp_token_custom_label: _, amm_pair_custom_label } => Ok(Response::new()),
                ExecuteMsg::AddAMMPairs { amm_pairs: _ } => Ok(Response::new())
            },
            BLOCK_SIZE,
        )
    }

    #[entry_point]
    pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
        pad_response_result(
            match (msg.id, msg.result) {
                (INSTANTIATE_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                    Some(x) => {                                             
                        let tempsss: Vec<u8> = x.to_vec().to_owned();                        
                        let temp = String::from_utf8_lossy(&tempsss);     
                        let mut temp: String = temp.to_string();
                        temp = temp.replace("(", "");
                        temp = temp.replace("\n", "");
                        let address = &temp[..40];
                        let contract_address = Addr::unchecked(address.clone());
                        let config = ephemeral_storage_r(deps.storage).load()?;
                        register_amm_pair(
                            deps.storage,
                            AMMPair {pair:config.pair,address:contract_address,enabled:true, code_hash: "".to_string() },
                        )?;
                        ephemeral_storage_w(deps.storage).remove();
                        Ok(Response::default())
                    }
                    None => Err(StdError::generic_err(format!("Expecting contract id"))),
                },
                _ => Err(StdError::generic_err(format!("Unknown reply id"))),
            },
            BLOCK_SIZE,
        )
    }

}
