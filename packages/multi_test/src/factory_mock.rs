
pub mod factory_mock{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary, CosmosMsg, BankMsg, Coin, Addr};
    use schemars::JsonSchema;
    use secret_multi_test::Contract;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{utils::{pad_query_result, pad_response_result}, factory::{ExecuteMsg, QueryMsg, QueryResponse}, core::{TokenType, ContractInstantiationInfo, Fee, ContractLink}, amm_pair::AMMSettings};
    use crate::util_addr::util_addr::OWNER;    
    pub const BLOCK_SIZE: usize = 256;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg{

    }
    

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        msg: InitMsg,
    ) -> StdResult<Response> {    
        Ok(Response::new())   
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::ListAMMPairs { pagination } => to_binary(""),
                QueryMsg::GetAMMPairAddress { pair } => to_binary(""),
                QueryMsg::GetConfig => to_binary(&QueryResponse::GetConfig { 
                    pair_contract: ContractInstantiationInfo{ code_hash: "".to_string(), id: 0u64 }, 
                    amm_settings: AMMSettings { 
                        lp_fee: Fee::new(3,100),
                        shade_dao_fee: Fee::new(3,100), 
                        shade_dao_address: ContractLink{ 
                            address: Addr::unchecked(OWNER),
                            code_hash:"".to_string()
                        }
                    }, 
                    lp_token_contract: ContractInstantiationInfo{ code_hash: "".to_string(), id: 0u64 }, 
                    authenticator: None 
                }),
                QueryMsg::GetAdmin => to_binary(""),
                QueryMsg::AuthorizeApiKey { api_key } => to_binary(""),
            },
            BLOCK_SIZE,
        )
    }

    
    #[entry_point]
    pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::SetConfig { pair_contract, lp_token_contract, amm_settings, api_key } =>  Ok(Response::new()),
                ExecuteMsg::CreateAMMPair { pair, entropy, staking_contract, router_contract } => Ok(Response::new()),
                ExecuteMsg::AddAMMPairs { amm_pairs } =>  Ok(Response::new()),
                ExecuteMsg::SetAdmin { admin } =>  Ok(Response::new()),
                ExecuteMsg::RegisterAMMPair { pair, signature } =>  Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }
}
