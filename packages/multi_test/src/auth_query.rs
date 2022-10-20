
pub mod auth_query{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary, CosmosMsg, BankMsg, Coin};
    use schemars::JsonSchema;
    use secret_multi_test::Contract;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{utils::pad_query_result, query_auth::{ExecuteMsg, QueryMsg, QueryAnswer}, core::TokenType};
    
    pub const BLOCK_SIZE: usize = 256;


    // #[entry_point]
    // pub fn instantiate(
    //     deps: DepsMut,
    //     env: Env,
    //     _info: MessageInfo,
    //     msg: InitMsg,
    // ) -> StdResult<Response> {
    
    //     Ok(Response::new())
    //         //(vec![
    //         // Attribute::new("staking_contract_addr", env.contract.address),
    //         // Attribute::new("reward_token", reward_token_address.address.to_string()),
    //         // Attribute::new("daily_reward_amount", msg.daily_reward_amount),
        
    // }

    // // #[entry_point]
    // // pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // //     pad_query_result(
    // //         match msg {
    // //             QueryMsg::Config {  } => todo!(),
    // //             QueryMsg::ValidateViewingKey { user, key } => {
    // //                 return to_binary(&QueryAnswer::ValidatePermit{
    // //                     sender:user;
    // //                     revoked = is_revoked;
    // //                 })
    // //             },
    // //             QueryMsg::ValidatePermit { permit } => todo!(),
    // //         },
    // //         BLOCK_SIZE,
    // //     )
    // // }

    
    // #[entry_point]
    // pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    //     pad_response_result(
    //         match msg {
    //             ExecuteMsg::SetAdminAuth { admin, padding } => todo!(),
    //             ExecuteMsg::SetRunState { state, padding } => todo!(),
    //             ExecuteMsg::SetViewingKey { key, padding } => todo!(),
    //             ExecuteMsg::CreateViewingKey { entropy, padding } => todo!(),
    //             ExecuteMsg::BlockPermitKey { key, padding } => todo!(),
    //         },
    //         BLOCK_SIZE,
    // )
}


