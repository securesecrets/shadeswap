pub const CONTRACT_ADDRESS: &str = "secret12qzz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const LP_TOKEN: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czczzz";
pub const REWARD_TOKEN: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy6";
pub const PROXY_STAKER_A: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy9";
pub const PROXY_STAKER_B: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy8";
pub const STAKER_A: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const STAKER_B: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";
pub const STAKER_C: &str = "secret1nulgwu6es24us9urgyvms7y02txyg0s02msgzw";
pub const SENDER: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmt2";

#[cfg(test)]
pub mod tests {
    use std::ops::Add;

    use query_authentication::{
        permit::Permit,
        transaction::{PermitSignature, PubKey},
    };
    use shadeswap_shared::{
        query_auth::PermitData,
        staking::{AuthQuery, ExecuteMsg, QueryData, QueryMsg, QueryResponse, RewardTokenInfo},
        utils::asset::Contract,
    };

    use super::*;
    use crate::{
        contract::{auth_queries, execute, query},
        operations::{
            calculate_staker_shares, claim_rewards, earned, generate_proxy_staking_key,
            get_user_claim_key, reward_per_token, set_reward_token, stake, unstake, update_reward,
        },
        query::{self},
        state::{
            claim_reward_info_r, config_w, proxy_staker_info_r, reward_token_list_r,
            reward_token_r, stakers_r, total_staked_r, total_staked_w, ClaimRewardsInfo, Config,
        },
        test::test_help_lib::{
            make_init_config, make_reward_token_contract, mock_custom_env, mock_dependencies,
            MockQuerier,
        },
    };
    use cosmwasm_std::{
        from_binary,
        testing::{mock_info, MockApi, MockStorage},
        to_binary, Addr, Binary, Decimal, MessageInfo, StdError, StdResult, Uint128,
    };

    use shadeswap_shared::utils::testing::assert_error;
    use shadeswap_shared::{c_std::OwnedDeps, core::TokenType};

    pub fn reward_token() -> TokenType {
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(REWARD_TOKEN),
            token_code_hash: REWARD_TOKEN.to_string(),
        }
    }

    #[test]
    fn assert_init_config() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        assert_eq!(config.daily_reward_amount, Uint128::from(100u128));
        assert_eq!(
            config.reward_token,
            TokenType::CustomToken {
                contract_addr: deps.as_mut().api.addr_validate(REWARD_TOKEN)?,
                token_code_hash: REWARD_TOKEN.to_string(),
            }
        );
        Ok(())
    }

    // total = 1500
    // amount = 500
    // share = 500/1500 = 0.33333333333333333
    #[test]
    fn assert_calculate_user_share_already_return_() -> StdResult<()> {
        let mut deps: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]).into();
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> =
            mock_dependencies(&[]).into();
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let _stake = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let user_shares =
            calculate_staker_shares(deps.as_mut().storage, Uint128::from(500u128)).unwrap();

        assert_eq!(Uint128::new(1500), total_staked_r(&deps.storage).load()?);
        assert_eq!(
            user_shares,
            Decimal::from_atomics(Uint128::new(333333333333333333), 18).unwrap()
        );
        Ok(())
    }

    #[test]
    fn assert_get_reward_token_list_success() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        set_reward_token(
            deps.as_mut(),
            &env,
            Uint128::new(100000000000u128),
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("REWARD_TOKEN_A".to_string()),
                token_code_hash: "".to_string(),
            },
            Uint128::new(30000000000000u128),
        )?;
        let auth_query = shadeswap_shared::staking::QueryMsg::GetRewardTokens {};
        let raw_response = query(deps.as_ref(), env, auth_query)?;
        let query_response: QueryResponse = from_binary(&raw_response)?;
        match query_response {
            QueryResponse::GetRewardTokens { tokens } => {
                assert_eq!(tokens.len(), 2);
            }
            _ => todo!(),
        };
        Ok(())
    }

    #[test]
    fn assert_proxy_stake_calculate_user_share_already_return() -> StdResult<()> {
        let mut deps: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]).into();
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> =
            mock_dependencies(&[]).into();
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let _stake = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let user_shares =
            calculate_staker_shares(deps.as_mut().storage, Uint128::from(500u128)).unwrap();
        assert_eq!(
            user_shares,
            Decimal::from_atomics(Uint128::new(333333333333333333), 18).unwrap()
        );
        Ok(())
    }

    #[test]
    fn assert_get_total_stakers_count_return_3() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let _env_b = mock_custom_env(CONTRACT_ADDRESS, 1571797854, 1534);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
        )?;
        let _stake_c = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1700u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
        )?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        Ok(())
    }

    //Test that when you proxy stake and stake it doesnt double count
    #[test]
    fn assert_proxy_stake_get_total_stakers_count_return_3() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let _env_b = mock_custom_env(CONTRACT_ADDRESS, 1571797854, 1534);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
        )?;
        let _stake_c = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1700u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_B)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
        )?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        Ok(())
    }

    #[test]
    fn assert_unstake_set_claimable_to_zero() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let unstake_mock_info = mock_info(STAKER_A, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
        )?;
        let _stake_c = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1700u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
        )?;
        let _unstake_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &unstake_mock_info.sender.clone(),
            Uint128::from(1000u128),
            Some(true),
        )?;
        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        let staker_info = stakers_r(deps.as_mut().storage)
            .load(&deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;
        assert_eq!(claim_reward_info_a.rewards, Uint128::zero());
        assert_eq!(staker_info.amount, Uint128::zero());
        Ok(())
    }

    #[test]
    fn assert_unstake_deduct_amount_from_total_stakes() -> StdResult<()> {
        type TestPermit = Permit<PermitData>;
        //secretd tx sign-doc file --from a
        let new_permit = TestPermit{
            params: PermitData { data: to_binary(&QueryData {}).unwrap(), key: "0".to_string()},
            chain_id: Some("secretdev-1".to_string()),
            sequence: Some(Uint128::zero()),
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(&"A07oJJ9n4TYTnD7ZStYyiPbB3kXOZvqIMkchGmmPRAzf".to_string()).unwrap()),
                signature: Binary::from_base64(&"bct9+cSJF+m51/be9/Bcc1zwfzYdMGzFMUH4VQl8EW9BuDDok6YEGzw6ZQOmu+rGqlFOfMBGybZbgINjD48rVQ==".to_string()).unwrap(),
            },
            account_number: Some(Uint128::zero()),
            memo: Some("".to_string())
        };
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let staker_a_addr: Addr = deps_owned
            .as_mut()
            .api
            .addr_validate("secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03")?;
        let _ = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &staker_a_addr.clone(),
            &staker_a_addr.clone(),
        )?;
        // Query With Permit StakerLpTokenInfo Staker A 1000
        let auth_query = QueryMsg::WithPermit {
            permit: new_permit.clone(),
            query: AuthQuery::GetStakerLpTokenInfo {},
        };
        let raw_response = query(deps.as_ref(), env.clone(), auth_query)?;
        let query_response: QueryResponse = from_binary(&raw_response)?;
        match query_response {
            QueryResponse::GetStakerLpTokenInfo {
                staked_lp_token,
                total_staked_lp_token,
            } => {
                assert_eq!(staked_lp_token, Uint128::from(1000u128));
                assert_eq!(total_staked_lp_token, Uint128::from(1000u128));
            }
            _ => {}
        }
        // UNSTAKE
        let msg_unstake = ExecuteMsg::Unstake {
            amount: Uint128::from(1000u128),
            remove_liquidity: Some(true),
        };
        let _ = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03", &[]),
            msg_unstake,
        )?;
        // Query With Permit StakerLpTokenInfo Staker A 0
        let auth_query = QueryMsg::WithPermit {
            permit: new_permit.clone(),
            query: AuthQuery::GetStakerLpTokenInfo {},
        };
        let raw_response = query(deps.as_ref(), env.clone(), auth_query)?;
        let query_response: QueryResponse = from_binary(&raw_response)?;
        match query_response {
            QueryResponse::GetStakerLpTokenInfo {
                staked_lp_token,
                total_staked_lp_token,
            } => {
                assert_eq!(staked_lp_token, Uint128::zero());
                assert_eq!(total_staked_lp_token, Uint128::zero());
            }
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn assert_unstake_higher_than_actual_amount_throws_exception() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let unstake_mock_info = mock_info(STAKER_A, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;

        let _unstake_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &unstake_mock_info.sender.clone(),
            Uint128::from(100000u128),
            Some(true),
        );

        match _unstake_a {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(
                StdError::generic_err("Unstaking Amount is higher then actual staking amount"),
                err
            ),
        }
        Ok(())
    }

    #[test]
    fn assert_unstake_non_staker_throws_exception() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let unstake_mock_info = mock_info(STAKER_A, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;

        stake(
            deps.as_mut(),
            &env,
            &mock_info(LP_TOKEN, &[]),
            Uint128::from(100000u128),
            &Addr::unchecked(STAKER_B),
            &Addr::unchecked(STAKER_B),
        )?;

        let _unstake_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &unstake_mock_info.sender.clone(),
            Uint128::from(100000u128),
            Some(true),
        );

        match _unstake_a {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(
                StdError::generic_err("Staking information does not exist"),
                err
            ),
        }
        Ok(())
    }

    #[test]
    fn assert_set_native_token_throws_exception() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let mut _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        _config.reward_token = TokenType::NativeToken {
            denom: "uscrt".to_string(),
        };
        config_w(&mut deps.storage).save(&_config)?;

        let error_msg = query::config(deps.as_ref());
        match error_msg {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(StdError::generic_err("Invalid reward token"), err),
        }
        Ok(())
    }

    #[test]
    fn assert_proxy_unstake_non_staker_throws_exception() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let unstake_mock_info = mock_info(STAKER_A, &[]);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &Addr::unchecked(STAKER_A),
            &Addr::unchecked(STAKER_B),
        )?;
        let _unstake_proxy_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &Addr::unchecked(STAKER_C),
            Uint128::from(100000u128),
            Some(false),
        );

        match _unstake_proxy_a {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(StdError::not_found("staking::state::StakingInfo"), err),
        }

        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &Addr::unchecked(STAKER_A),
            &Addr::unchecked(STAKER_B),
        )?;
        let _unstake_proxy_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &Addr::unchecked(STAKER_B),
            Uint128::from(100000u128),
            Some(false),
        );

        match _unstake_proxy_a {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(
                StdError::generic_err("Unstaking Amount is higher then actual staking amount"),
                err
            ),
        }
        Ok(())
    }

    #[test]
    fn assert_proxy_stake_with_wrong_caller_throws_exception() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(STAKER_B, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &Addr::unchecked(STAKER_A),
            &Addr::unchecked(STAKER_C),
        );
        match _stake_a {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(StdError::generic_err("Token sent is not LP Token."), err),
        }
        Ok(())
    }

    #[test]
    fn assert_stake_with_wrong_caller_throws_exception() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(STAKER_B, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &Addr::unchecked(STAKER_A),
            &Addr::unchecked(STAKER_A),
        );
        match _stake_a {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(StdError::generic_err("Token sent is not LP Token."), err),
        }
        Ok(())
    }
    #[test]
    fn assert_proxy_unstake_set_claimable_to_zero() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let unstake_mock_info = mock_info(PROXY_STAKER_A, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
        )?;
        let _stake_c = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1700u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
        )?;
        let _unstake_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            Uint128::from(1000u128),
            None,
        )?;

        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        let staker_info = stakers_r(deps.as_mut().storage)
            .load(&deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;

        let proxy_staker_info =
            proxy_staker_info_r(deps.as_mut().storage).load(&generate_proxy_staking_key(
                &&deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
                &&deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            ))?;
        assert_eq!(staker_info.amount, Uint128::zero());
        assert_eq!(proxy_staker_info.amount, Uint128::zero());
        assert_eq!(claim_reward_info_a.rewards, Uint128::zero());
        Ok(())
    }

    #[test]
    fn assert_staking_with_claim_rewards() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 1500000);
        let env_b = mock_custom_env(CONTRACT_ADDRESS, 1534, 1600000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(300000u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        // STAKER A
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap(),
            &deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap(),
        )
        .unwrap();
        // assert there is no claim reward for staker a
        let claim_reward_info = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )
            .unwrap();

        // Claimable for staker A throws Exception
        assert_eq!(claim_reward_info.rewards, Uint128::zero());
        let total_staked_amount = total_staked_r(deps.as_mut().storage).load().unwrap();
        assert_eq!(total_staked_amount, Uint128::new(1000u128));

        // STAKER B
        let _stake_b = stake(
            deps.as_mut(),
            &env_b,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
        )?;

        update_reward(
            Uint128::new(1600000),
            &deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap(),
            deps.as_mut().storage,
            &env,
        )?;

        // Assert Staker A claimable reward
        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        assert_eq!(claim_reward_info_a.rewards, Uint128::new(300000u128));

        // Claimable for staker B throws Exception
        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage)
            .load(deps_owned.as_mut().api.addr_validate(STAKER_B)?.as_bytes());
        // Claimable for staker A throws Exception
        match claim_reward_info_b {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(err, StdError::not_found("staking::state::ClaimRewardsInfo")),
        }

        // timestamp 1600000000
        claim_rewards(
            deps.as_mut(),
            Uint128::new(1600000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &env_b,
        )?;
        claim_rewards(
            deps.as_mut(),
            Uint128::new(1600000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &env_b,
        )?;

        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        let claim_reward_info_b: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_B)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        assert_eq!(claim_reward_info_a.rewards, Uint128::new(0u128));
        assert_eq!(claim_reward_info_b.rewards, Uint128::new(0u128));
        let staker_info_a = stakers_r(deps.as_mut().storage)
            .load(&deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;
        let staker_info_b = stakers_r(deps.as_mut().storage)
            .load(deps_owned.as_mut().api.addr_validate(STAKER_B)?.as_bytes())?;

        assert_eq!(staker_info_a.amount, Uint128::new(1000));
        assert_eq!(staker_info_b.amount, Uint128::new(1500));

        // move timestamp 1700000000

        claim_rewards(
            deps.as_mut(),
            Uint128::new(1700000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &env_b,
        )?;
        claim_rewards(
            deps.as_mut(),
            Uint128::new(1700000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &env_b,
        )?;

        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        let claim_reward_info_b: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_B)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;
        assert_eq!(claim_reward_info_a.rewards, Uint128::new(0u128));
        assert_eq!(claim_reward_info_b.rewards, Uint128::new(0u128));
        Ok(())
    }

    #[test]
    fn assert_calculation() -> StdResult<()> {
        assert_eq!(
            reward_per_token(
                Uint128::new(16000000u128),
                &RewardTokenInfo {
                    reward_token: reward_token(),
                    reward_rate: Uint128::new(3),
                    valid_to: Uint128::new(17000000u128),
                    reward_per_token_stored: Uint128::new(0),
                    last_update_time: Uint128::new(15000000u128)
                },
                Uint128::new(1000u128)
            )?,
            Uint128::new(3000000000000000000000u128)
        );
        Ok(())
    }

    // const reward_amount = 300000
    // ration is 0.4 : 0.6 (Staker_A : Staker_B)
    // seconds 86,400,000
    // stake -> staker_a 15000000time -> 1000amount
    // stake -> staker_b 16000000time -> 1500amount
    // 1. (3 * 1000000) => 300000-> Staker A
    //
    #[test]
    fn assert_calculate_staking_reward() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 15000000);
        let env_b = mock_custom_env(CONTRACT_ADDRESS, 1600, 16000000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(300000u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env_b,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_ref().api.addr_validate(STAKER_B)?,
            &deps_owned.as_ref().api.addr_validate(STAKER_B)?,
        )?;
        // let user_shares =
        //     calculate_staker_shares(deps.as_mut().storage, Uint128::from(1000u128)).unwrap();

        update_reward(
            Uint128::from(16000000u128),
            &&deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            deps.as_mut().storage,
            &env_b,
        )?;

        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;
        assert_eq!(claim_reward_info_a.rewards, Uint128::from(3000000u128));
        Ok(())
    }

    #[test]
    fn assert_calculate_proxy_staking_reward() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 15000000);
        let env_b = mock_custom_env(CONTRACT_ADDRESS, 1534, 16000000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(300000u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env_b,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_ref().api.addr_validate(STAKER_B)?,
            &deps_owned.as_ref().api.addr_validate(STAKER_B)?,
        )?;

        let reward_tokens = reward_token_list_r(deps.as_mut().storage).load().unwrap();
        let reward_token_info: RewardTokenInfo = reward_token_r(deps.as_mut().storage)
            .load(reward_tokens[0].to_owned().as_bytes())
            .unwrap();
        update_reward(
            Uint128::from(16000000u128),
            &&deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            deps.as_mut().storage,
            &env,
        )?;

        let claim_reward_info_a: ClaimRewardsInfo = claim_reward_info_r(deps.as_mut().storage)
            .load(
                get_user_claim_key(
                    deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                    reward_token().unique_key(),
                )
                .as_bytes(),
            )?;

        let staker_info = stakers_r(deps.as_ref().storage)
            .load(&deps_owned.as_mut().api.addr_validate(STAKER_A)?.as_bytes())?;
        let staking_reward = earned(
            staker_info.amount,
            reward_token_info.reward_per_token_stored,
            claim_reward_info_a.reward_token_per_token_paid,
            claim_reward_info_a.rewards,
        )?;
        assert_eq!(staking_reward, Uint128::from(3000000u128));
        Ok(())
    }

    #[test]
    fn assert_staking_first_time_store_timestamp() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 15000000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(300000u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap(),
            &deps_owned.as_mut().api.addr_validate(STAKER_A).unwrap(),
        )
        .unwrap();
        update_reward(
            Uint128::from(16000000u128),
            &&deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            deps.as_mut().storage,
            &env,
        )?;

        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage).load(
            deps_owned
                .as_mut()
                .api
                .addr_validate(STAKER_B)
                .unwrap()
                .as_bytes(),
        );
        match claim_reward_info_b {
            Ok(_) => todo!(),
            Err(err) => assert_eq!(err, StdError::not_found("staking::state::ClaimRewardsInfo")),
        }
        Ok(())
    }

    #[test]
    fn assert_proxy_unstaking_errors() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1571797523, 1524);
        let stake_mock_info = mock_info(LP_TOKEN, &[]);
        let unstake_mock_info = mock_info(PROXY_STAKER_A, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(100u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
        )?;
        let _stake_c = stake(
            deps.as_mut(),
            &env,
            &stake_mock_info,
            Uint128::from(1700u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_C)?,
        )?;

        //Check you cannot unstake what you did not stake
        assert_error(
            unstake(
                deps.as_mut(),
                &env,
                &unstake_mock_info.sender.clone(),
                &deps_owned.as_mut().api.addr_validate(STAKER_B)?,
                Uint128::from(1000u128),
                None,
            ),
            "Proxy stake for given proxy staker and staker does not exist.".to_string(),
        );

        //Check you cannot unstake more then you staked
        assert_error(
            unstake(
                deps.as_mut(),
                &env,
                &deps_owned.as_mut().api.addr_validate(PROXY_STAKER_A)?,
                &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
                Uint128::from(1001u128),
                None,
            ),
            "Unstaking Amount is higher then actual staking amount".to_string(),
        );

        //Check you cannot unstake a proxy staked amount
        assert_error(
            unstake(
                deps.as_mut(),
                &env,
                &unstake_mock_info.sender.clone(),
                &unstake_mock_info.sender.clone(),
                Uint128::from(1000u128),
                Some(true),
            ),
            "Staking information does not exist".to_string(),
        );

        let _unstake_a = unstake(
            deps.as_mut(),
            &env,
            &unstake_mock_info.sender.clone(),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            Uint128::from(1000u128),
            None,
        )?;
        Ok(())
    }

    #[test]
    fn assert_staking_last_time_claim_less_than_valid_to_current_timestamp_less_than_valid_to(
    ) -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let env = mock_custom_env(LP_TOKEN, 1500, 16000000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::from(300000u128))?;
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1000u128),
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::from(1500u128),
            &deps_owned.as_ref().api.addr_validate(STAKER_B)?,
            &deps_owned.as_ref().api.addr_validate(STAKER_B)?,
        )?;
        let current_timestamp = Uint128::from(17000000u128);

        claim_rewards(
            deps.as_mut(),
            current_timestamp,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &env,
        )?;

        claim_rewards(
            deps.as_mut(),
            current_timestamp,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &env,
        )?;
        let claim_reward_info_a = claim_reward_info_r(deps.as_mut().storage).load(
            get_user_claim_key(
                deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                reward_token().unique_key(),
            )
            .as_bytes(),
        )?;

        update_reward(
            current_timestamp,
            &&deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            deps.as_mut().storage,
            &env,
        )?;
        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage).load(
            get_user_claim_key(
                deps_owned.as_mut().api.addr_validate(STAKER_B)?.to_string(),
                reward_token().unique_key(),
            )
            .as_bytes(),
        )?;
        assert_eq!(claim_reward_info_a.rewards, Uint128::new(0u128));
        assert_eq!(claim_reward_info_b.rewards, Uint128::new(1800000u128));
        Ok(())
    }

    // reward = 300000
    // last_timestamp = 16000000
    // current_timestamp = 21000000
    // valid_to = 19000000
    // 17000000 - 16000000 = (100000 * 300000) / 86400 * 0.4 =
    // 19000000 - 17000000 = (200000 * 300000) / 86400 * 0.4 =
    // 1.  4166665 -> Staker A
    #[test]
    fn assert_staking_last_time_claim_less_than_valid_to_current_timestamp_higher_than_valid_to(
    ) -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 16000000);
        let mock_info = mock_info(LP_TOKEN, &[]);
        let staker_a = &deps_owned.as_mut().api.addr_validate(STAKER_A)?;
        let staker_b = deps_owned.as_mut().api.addr_validate(STAKER_B)?;
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::new(300000u128))?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::new(1000u128),
            &staker_a,
            &staker_a,
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &mock_info,
            Uint128::new(1500u128),
            &staker_b,
            &staker_b,
        )?;
        set_reward_token(
            deps.as_mut(),
            &mock_custom_env(CONTRACT_ADDRESS, 15834, 17000000),
            Uint128::new(300000u128),
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(CONTRACT_ADDRESS),
                token_code_hash: "".to_string(),
            },
            Uint128::new(19000000u128),
        )?;
        let current_timestamp = Uint128::from(17000000u128);

        claim_rewards(
            deps.as_mut(),
            current_timestamp,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &env,
        )?;
        claim_rewards(
            deps.as_mut(),
            current_timestamp,
            &deps_owned.as_mut().api.addr_validate(STAKER_A)?,
            &env,
        )?;
        let claim_reward_info_a = claim_reward_info_r(deps.as_mut().storage).load(
            get_user_claim_key(
                deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                reward_token().unique_key(),
            )
            .as_bytes(),
        )?;
        update_reward(current_timestamp, &staker_b, deps.as_mut().storage, &env)?;
        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage).load(
            get_user_claim_key(
                deps_owned.as_mut().api.addr_validate(STAKER_B)?.to_string(),
                reward_token().unique_key(),
            )
            .as_bytes(),
        )?;
        let staker_info_a = stakers_r(deps.as_mut().storage).load(staker_a.clone().as_bytes())?;
        let staker_info_b = stakers_r(deps.as_mut().storage).load(staker_b.clone().as_bytes())?;
        assert_eq!(staker_info_a.amount, Uint128::new(1000));
        assert_eq!(staker_info_b.amount, Uint128::new(1500));
        assert_eq!(claim_reward_info_a.rewards, Uint128::new(0));
        assert_eq!(claim_reward_info_b.rewards, Uint128::new(1800000u128));
        Ok(())
    }

    /**
     * 3000000 seconds pass
     * Ratio is 0.6/0.4
     * 0.6 * 3000000 * 3 = 5400000
     */
    #[test]
    fn assert_claim_reward_no_change_last_time_reward_info() -> StdResult<()> {
        let mut deps = mock_dependencies(&[]);
        let mut deps_owned: OwnedDeps<MockStorage, MockApi, MockQuerier> = mock_dependencies(&[]);
        let env = mock_custom_env(CONTRACT_ADDRESS, 1500, 16000000);
        let mock_info_lp_token = mock_info(LP_TOKEN, &[]);
        let staker_a = &deps_owned.as_mut().api.addr_validate(STAKER_A)?;
        let staker_b = deps_owned.as_mut().api.addr_validate(STAKER_B)?;
        let _config: Config = make_init_config(deps.as_mut(), &env, Uint128::new(300000u128))?;
        let _stake_a = stake(
            deps.as_mut(),
            &env,
            &mock_info_lp_token,
            Uint128::new(1000u128),
            &staker_a.clone(),
            &staker_a.clone(),
        )?;
        let _stake_b = stake(
            deps.as_mut(),
            &env,
            &mock_info_lp_token,
            Uint128::new(1500u128),
            &staker_b,
            &staker_b,
        )?;
        set_reward_token(
            deps.as_mut(),
            &mock_custom_env(CONTRACT_ADDRESS, 15834, 17000000),
            Uint128::new(300000u128),
            reward_token(),
            Uint128::new(19000000u128),
        )?;
        let current_timestamp = Uint128::from(21000000u128);
        claim_rewards(
            deps.as_mut(),
            current_timestamp,
            &staker_a,
            &mock_custom_env(&CONTRACT_ADDRESS, 1600, 21000000),
        )?;
        let claim_reward_info_a = claim_reward_info_r(deps.as_mut().storage).load(
            get_user_claim_key(
                deps_owned.as_mut().api.addr_validate(STAKER_A)?.to_string(),
                reward_token().unique_key(),
            )
            .as_bytes(),
        )?;

        update_reward(
            Uint128::new(21000000u128),
            &&deps_owned.as_mut().api.addr_validate(STAKER_B)?,
            deps.as_mut().storage,
            &env,
        )?;

        let claim_reward_info_b = claim_reward_info_r(deps.as_mut().storage).load(
            get_user_claim_key(staker_b.to_string(), reward_token().unique_key()).as_bytes(),
        )?;
        assert_eq!(claim_reward_info_a.rewards, Uint128::zero());
        assert_eq!(claim_reward_info_b.rewards, Uint128::new(5400000u128));
        Ok(())
    }
}

#[cfg(test)]
pub mod test_help_lib {
    use super::*;
    use cosmwasm_std::{
        from_slice,
        testing::{mock_info, MockApi, MockStorage},
        to_binary, Addr, BalanceResponse, BlockInfo, Coin, ContractInfo, DepsMut, Empty, Env,
        OwnedDeps, Querier, QuerierResult, QueryRequest, StdResult, Timestamp, TransactionInfo,
        Uint128,
    };
    use serde::{Deserialize, Serialize};
    use shadeswap_shared::{
        admin::ValidateAdminPermissionResponse,
        snip20::{manager::Balance, QueryAnswer},
        staking::InitMsg,
        Contract,
    };

    use crate::{
        contract::instantiate,
        state::{config_r, config_w, Config},
        test::tests::reward_token,
    };

    pub fn make_reward_token_contract(address: &str, code_hash: &str) -> StdResult<Contract> {
        let mut deps = mock_dependencies(&[]);
        return Ok(Contract {
            address: deps.as_mut().api.addr_validate(address)?,
            code_hash: code_hash.to_string(),
        });
    }

    pub fn make_init_config(
        mut deps: DepsMut,
        env: &Env,
        daily_reward_amount: Uint128,
    ) -> StdResult<Config> {
        let info = mock_info(SENDER, &[]);
        let msg = InitMsg {
            daily_reward_amount: daily_reward_amount.clone(),
            reward_token: reward_token(),
            pair_contract: Contract {
                address: deps.api.addr_validate(CONTRACT_ADDRESS)?,
                code_hash: "".to_string().clone(),
            },
            prng_seed: to_binary(&"prng")?,
            lp_token: Contract {
                address: Addr::unchecked("".to_string()),
                code_hash: "".to_string(),
            },
            authenticator: None,
            admin_auth: Contract {
                address: Addr::unchecked("admin"),
                code_hash: "".to_string(),
            },
            valid_to: Uint128::new(3747905010000u128),
        };
        assert!(instantiate(deps.branch(), env.clone(), info.clone(), msg).is_ok());
        let mut config = config_r(deps.storage).load()?;
        config.lp_token = Contract {
            address: deps.api.addr_validate(LP_TOKEN)?,
            code_hash: "".to_string(),
        };
        config_w(deps.storage).save(&config)?;
        Ok(config)
    }

    pub fn mock_custom_env(address: &str, height: u64, time: u64) -> Env {
        Env {
            block: BlockInfo {
                height,
                time: Timestamp::from_seconds(time),
                chain_id: "pulsar-2".to_string(),
            },
            transaction: Some(TransactionInfo { index: 3 }),
            contract: ContractInfo {
                address: Addr::unchecked(address),
                code_hash: "".to_string(),
            },
        }
    }

    pub fn mock_dependencies(
        _contract_balance: &[Coin],
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier { portion: 100 },
            custom_query_type: std::marker::PhantomData,
        }
    }

    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
    }

    pub struct MockQuerier {
        portion: u128,
    }

    impl Querier for MockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
            match &request {
                QueryRequest::Bank(msg) => match msg {
                    cosmwasm_std::BankQuery::Balance { address, denom: _ } => {
                        match address.as_str() {
                            _custom_token_2 => {
                                let balance = to_binary(&QueryAnswer::Balance {
                                    amount: Uint128::from(1000u128),
                                })
                                .unwrap();
                                QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                            }
                            "admin" => QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                                to_binary(&ValidateAdminPermissionResponse {
                                    has_permission: true,
                                })
                                .unwrap(),
                            )),
                            _factory_contract_address => {
                                let balance = to_binary(&BalanceResponse {
                                    amount: Coin {
                                        denom: "uscrt".into(),
                                        amount: Uint128::from(1000000u128),
                                    },
                                })
                                .unwrap();
                                QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                            }
                            _custom_token_1 => {
                                let balance = to_binary(&BalanceResponse {
                                    amount: Coin {
                                        denom: "uscrt".into(),
                                        amount: Uint128::from(1000000u128),
                                    },
                                })
                                .unwrap();
                                QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                            }
                            _ => {
                                let response: &str = &address.to_string();
                                unimplemented!("{} not implemented", address.as_str())
                            }
                        }
                    }
                    cosmwasm_std::BankQuery::AllBalances { address: _ } => todo!(),
                    _ => todo!(),
                },
                QueryRequest::Custom(_) => todo!(),
                QueryRequest::Wasm(msg) => match msg {
                    cosmwasm_std::WasmQuery::Smart {
                        contract_addr,
                        code_hash: _,
                        msg: _,
                    } => match contract_addr.as_str() {
                        _custom_token_1 => {
                            let balance = to_binary(&QueryAnswer::Balance {
                                amount: Uint128::from(10000u128),
                            })
                            .unwrap();
                            QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                        }
                        _custom_token_2 => {
                            let balance = to_binary(&QueryAnswer::Balance {
                                amount: Uint128::from(10000u128),
                            })
                            .unwrap();
                            QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(balance))
                        }
                        _ => {
                            unimplemented!("{} not implemented", contract_addr.as_str().to_owned())
                        }
                    },
                    cosmwasm_std::WasmQuery::ContractInfo { contract_addr: _ } => {
                        unimplemented!("unimplemented")
                    }
                    cosmwasm_std::WasmQuery::Raw {
                        key: _,
                        contract_addr: _,
                    } => unimplemented!("unimplemented"),
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
    }
}
