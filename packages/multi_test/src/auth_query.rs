
pub mod auth_query{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::utils::pad_query_result;
    
    pub const BLOCK_SIZE: usize = 256;

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    pub struct InitMsg {
            
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    pub enum QueryMsg {
        GetDummy{}
    }

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        msg: InitMsg,
    ) -> StdResult<Response> {
    
        Ok(Response::new())
            //(vec![
            // Attribute::new("staking_contract_addr", env.contract.address),
            // Attribute::new("reward_token", reward_token_address.address.to_string()),
            // Attribute::new("daily_reward_amount", msg.daily_reward_amount),
        
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::GetDummy {} => to_binary(&"".to_string())
            },
            BLOCK_SIZE,
        )
    }

}