pub mod staking_mock {
    use cosmwasm_std::{
        entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    };
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{
        staking::{ExecuteMsg, QueryMsg},
        utils::{pad_query_result, pad_response_result},
    };

    pub const BLOCK_SIZE: usize = 256;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {}

    #[entry_point]
    pub fn instantiate(
        _deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        _msg: InitMsg,
    ) -> StdResult<Response> {
        println!("test init staking");
        let mut response = Response::new();
        response.data = Some(env.contract.address.as_bytes().into());
        Ok(response)
    }

    #[entry_point]
    pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        pad_query_result(
            match msg {
                QueryMsg::GetConfig {} => to_binary(""),
                QueryMsg::WithPermit {
                    permit: _,
                    query: _,
                } => to_binary(""),
                QueryMsg::GetRewardTokens {} => to_binary("")
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
                ExecuteMsg::ClaimRewards {} => Ok(Response::new()),
                ExecuteMsg::ProxyUnstake {
                    for_addr: _,
                    amount: _,
                } => Ok(Response::new()),
                ExecuteMsg::Unstake {
                    amount: _,
                    remove_liqudity: _,
                } => Ok(Response::new()),
                ExecuteMsg::Receive {
                    from: _,
                    msg: _,
                    amount: _,
                } => Ok(Response::new()),
                ExecuteMsg::SetRewardToken {
                    reward_token: _,
                    daily_reward_amount: _,
                    valid_to: _,
                } => Ok(Response::new()),
                ExecuteMsg::SetAuthenticator { authenticator: _ } => Ok(Response::new()),
                ExecuteMsg::RecoverFunds {
                    token: _,
                    amount: _,
                    to: _,
                    msg: _,
                } => Ok(Response::new()),
                ExecuteMsg::SetConfig { admin_auth: _ } => todo!(),
            },
            BLOCK_SIZE,
        )
    }
}
