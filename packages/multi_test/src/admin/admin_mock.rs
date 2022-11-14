pub mod admin_mock{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary};
    use schemars::JsonSchema;
    use serde::{Serialize, Deserialize};
    use shadeswap_shared::{utils::{pad_query_result, pad_response_result}, admin::{ExecuteMsg, QueryMsg, ValidateAdminPermissionResponse}};

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
                QueryMsg::GetConfig {  } =>  to_binary(""),
                QueryMsg::GetAdmins {  } =>  to_binary(""),
                QueryMsg::GetPermissions { user: _ } =>  to_binary(""),
                QueryMsg::ValidateAdminPermission { permission: _, user:_ } => to_binary(&ValidateAdminPermissionResponse{ 
                    has_permission: true })
            },
            BLOCK_SIZE,
        )
    }

    
    #[entry_point]
    pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::UpdateRegistry { action: _ } =>  Ok(Response::new()),
                ExecuteMsg::UpdateRegistryBulk { actions: _ } =>  Ok(Response::new()),
                ExecuteMsg::TransferSuper { new_super:_ } => Ok(Response::new()),
                ExecuteMsg::SelfDestruct {  } =>  Ok(Response::new()),
                ExecuteMsg::ToggleStatus { new_status :_ } => Ok(Response::new()),
            },
            BLOCK_SIZE,
        )
    }
}


