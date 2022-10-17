
pub mod auth_query{

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        msg: InitMsg,
    ) -> StdResult<Response> {
    
        Ok(Response::new()
            //(vec![
            // Attribute::new("staking_contract_addr", env.contract.address),
            // Attribute::new("reward_token", reward_token_address.address.to_string()),
            // Attribute::new("daily_reward_amount", msg.daily_reward_amount),
        )
    }

    // #[entry_point]
    // pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    //     pad_query_result(
    //         match msg {
    //             QueryMsg::GetConfig {} => get_config(deps),
    //             QueryMsg::GetContractOwner {} => todo!(),
    //             QueryMsg::WithPermit { permit, query } => {
    //                 let config = config_r(deps.storage).load()?;
    //                 let res: PermitAuthentication<QueryData> =
    //                     authenticate_permit(deps, permit, &deps.querier, config.authenticator)?;

    //                 if res.revoked {
    //                     return Err(StdError::generic_err("".to_string()));
    //                 }

    //                 auth_queries(deps, env, query, res.sender)
    //             }
    //             QueryMsg::GetAdmin {} => to_binary(&QueryResponse::GetAdmin {
    //                 admin: admin_r(deps.storage).load()?,
    //             }),
    //         },
    //         BLOCK_SIZE,
    //     )
    // }

}