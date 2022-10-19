
pub mod auth_query{
    use cosmwasm_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point, to_binary, Deps, Binary, CosmosMsg, BankMsg, Coin};
    use schemars::JsonSchema;
    use secret_multi_test::Contract;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{utils::pad_query_result, query_auth::ExecuteMsg, core::TokenType};
    
    pub const BLOCK_SIZE: usize = 256;

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    pub struct InitMsg {
            
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    pub struct ExecuteMsg {
            
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

    
    #[entry_point]
    pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
        pad_response_result(
            match msg {
                ExecuteMsg::ProxyUnstake { for_addr, amount } => {
                    proxy_unstake(deps, env, info, for_addr, amount)
                },
                ExecuteMsg::Receive {
                    from, amount, msg, ..
                } => receiver_callback(deps, env, info, from, amount, msg),
                ExecuteMsg::ClaimRewards {} => claim_rewards(deps, info, env),
                ExecuteMsg::Unstake {
                    amount,
                    remove_liqudity,
                } => unstake(deps, env, info, amount, remove_liqudity),
                ExecuteMsg::SetAuthenticator { authenticator } => {
                    apply_admin_guard(&info.sender, deps.storage)?;
                    update_authenticator(deps.storage, authenticator)
                }
                ExecuteMsg::SetAdmin { admin } => {
                    apply_admin_guard(&info.sender, deps.storage)?;
                    admin_w(deps.storage).save(&admin)?;
                    Ok(Response::default())
                }
                ExecuteMsg::SetRewardToken {
                    reward_token,
                    daily_reward_amount,
                    valid_to,
                } => {
                    apply_admin_guard(&info.sender, deps.storage)?;
                    set_reward_token(deps, env, info, reward_token, daily_reward_amount, valid_to)
                },
                ExecuteMsg::RecoverFunds {
                    token,
                    amount,
                    to,
                    msg,
                } => {
                    apply_admin_guard(&info.sender, deps.storage)?;
                    let send_msg = match token {
                        TokenType::CustomToken { contract_addr, token_code_hash } => vec![send_msg(
                            to,
                            amount,
                            msg,
                            None,
                            None,
                            &Contract{
                                address: contract_addr,
                                code_hash: token_code_hash
                            }
                        )?],
                        TokenType::NativeToken { denom } => vec![CosmosMsg::Bank(BankMsg::Send {
                            to_address: to.to_string(),
                            amount: vec![Coin::new(amount.u128(), denom)],
                        })],
                    };

                    Ok(Response::new().add_messages(send_msg))
                }
            },
            BLOCK_SIZE,
    )
}


}