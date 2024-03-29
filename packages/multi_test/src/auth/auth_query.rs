pub mod auth_query{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary};
    use query_authentication::transaction::{PubKeyValue};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{utils::{pad_query_result, pad_response_result}, query_auth::{ExecuteMsg, QueryMsg, QueryAnswer}};
    
    pub const BLOCK_SIZE: usize = 256;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg{

    }
    

    #[entry_point]
    pub fn instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: InitMsg,
    ) -> StdResult<Response> {    
        Ok(Response::new())   
    }

    #[entry_point]
    pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::Config {  } => to_binary(""),
                QueryMsg::ValidateViewingKey { user:_, key:_ } => to_binary(""),
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
    pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::SetAdminAuth { admin: _, padding: _ } => Ok(Response::new()),
                ExecuteMsg::SetRunState { state: _, padding: _ } => Ok(Response::new()),
                ExecuteMsg::SetViewingKey { key: _, padding: _ } => Ok(Response::new()),
                ExecuteMsg::CreateViewingKey { entropy: _, padding: _ } => Ok(Response::new()),
                ExecuteMsg::BlockPermitKey { key: _, padding: _ } => Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }
}


