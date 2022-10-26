
pub mod auth_query{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary, CosmosMsg, BankMsg, Coin, Addr};
    use query_authentication::transaction::{PubKey, PubKeyValue};
    use schemars::JsonSchema;
    use secret_multi_test::Contract;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{utils::{pad_query_result, pad_response_result}, query_auth::{ExecuteMsg, QueryMsg, QueryAnswer}, core::TokenType};

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
                QueryMsg::Config {  } => to_binary(""),
                QueryMsg::ValidateViewingKey { user, key } => to_binary(""),
                QueryMsg::ValidatePermit { permit } => {
                    let pub_key = permit.signature.pub_key.value.clone(); 
                    let pub_key_value = PubKeyValue(pub_key);                  
                    println!(" Mock Validating Permit for Addr {}", pub_key_value.as_addr(None)?);
                    return to_binary(&QueryAnswer::ValidatePermit { 
                        user:  pub_key_value.as_addr(None)?, 
                        is_revoked: false 
                    });
                }
            },
            BLOCK_SIZE,
        )
    }

    
    #[entry_point]
    pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::SetAdminAuth { admin, padding } => Ok(Response::new()),
                ExecuteMsg::SetRunState { state, padding } => Ok(Response::new()),
                ExecuteMsg::SetViewingKey { key, padding } => Ok(Response::new()),
                ExecuteMsg::CreateViewingKey { entropy, padding } => Ok(Response::new()),
                ExecuteMsg::BlockPermitKey { key, padding } => Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }
}


