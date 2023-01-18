use cosmwasm_std::{to_binary, Addr, ContractInfo, Empty};
use multi_test::help_lib::integration_help_lib::mk_address;
use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::staking::{ExecuteMsg, InitMsg, QueryResponse};
use staking::contract::{execute, instantiate, query};

use shadeswap_shared::utils::asset::Contract as AuthContract;
pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
}
use crate::staking_help_query::query_claimable_reward;
use cosmwasm_std::{Coin, StdError, Uint128};
use multi_test::admin::admin_help::init_admin_contract;
use multi_test::help_lib::integration_help_lib::send_snip20_to_stake;
use multi_test::help_lib::integration_help_lib::{
    convert_to_contract_link, get_current_block_time, mint_deposit_snip20, roll_blockchain,
    send_snip20_to_proxy_stake, set_viewing_key, store_init_auth_contract,
};
use multi_test::help_lib::integration_help_lib::{generate_snip20_contract, snip_20_balance_query};
use multi_test::util_addr::util_addr::{OWNER, OWNER_PUB_KEY, PUB_KEY_STAKER_A, STAKER_A};
use multi_test::util_addr::util_blockchain::CHAIN_ID;
use shadeswap_shared::core::TokenType;
use shadeswap_shared::staking::QueryMsg;
use shadeswap_shared::utils::testing::TestingExt;
use shadeswap_shared::Contract as SContract;

pub fn set_up_tests(
    router: &mut App,
    owner_addr: &Addr,
) -> (ContractInfo, ContractInfo, ContractInfo) {
    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &owner_addr.clone(),
                vec![Coin {
                    denom: "uscrt".into(),
                    amount: Uint128::new(100000000000000u128),
                }],
            )
            .unwrap();
    });

    router.block_info().chain_id = CHAIN_ID.to_string();
    roll_blockchain(router, 1).unwrap();
    let admin_contract = init_admin_contract(router, &owner_addr).unwrap();
    let reward_contract =
        generate_snip20_contract(router, "RWD".to_string(), "RWD".to_string(), 18).unwrap();
    let staking_contract_info = router.store_code(staking_contract_store());
    let auth_contract = store_init_auth_contract(router).unwrap();
    let lp_token_contract =
        generate_snip20_contract(router, "LPT".to_string(), "LPT".to_string(), 18).unwrap();
    let init_msg = InitMsg {
        daily_reward_amount: Uint128::new(300000u128),
        reward_token: TokenType::CustomToken {
            contract_addr: reward_contract.address.to_owned(),
            token_code_hash: reward_contract.code_hash.to_owned(),
        },
        pair_contract: SContract {
            address: Addr::unchecked("AMMPAIR"),
            code_hash: "".to_string(),
        },
        prng_seed: to_binary(&"password").unwrap(),
        lp_token: SContract {
            address: lp_token_contract.address.to_owned(),
            code_hash: lp_token_contract.code_hash.to_owned(),
        },
        authenticator: Some(AuthContract {
            address: auth_contract.address.to_owned(),
            code_hash: auth_contract.code_hash.to_owned(),
        }),
        admin_auth: convert_to_contract_link(&admin_contract),
        valid_to: Uint128::new(3747905010000u128),
    };

    let staking_contract = router
        .instantiate_contract(
            staking_contract_info,
            mk_address(&OWNER).to_owned(),
            &init_msg,
            &[],
            "staking",
            Some(OWNER.to_string()),
        )
        .unwrap();

    (staking_contract, reward_contract, lp_token_contract)
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests() {
    use multi_test::help_lib::integration_help_lib::get_snip20_balance;

    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());
    let owner_addr = Addr::unchecked(OWNER);
    let mut router = App::default();

    let (staking_contract, reward_contract, lp_token_contract) =
        set_up_tests(&mut router, &owner_addr);

    roll_blockchain(&mut router, 2).unwrap();

    // Assert Staking Config
    let query: QueryResponse = router
        .query_test(
            staking_contract.to_owned(),
            to_binary(&QueryMsg::GetConfig {}).unwrap(),
        )
        .unwrap();
    match query {
        QueryResponse::GetConfig {
            reward_token,
            lp_token,
            amm_pair: _,
            admin_auth: _,
            total_staked_lp_token,
        } => {
            assert_eq!(
                reward_token.address.to_string(),
                reward_contract.address.to_string()
            );
            assert_eq!(
                lp_token.address.to_owned(),
                lp_token_contract.address.to_owned()
            );
        }
        _ => panic!("Query Responsedoes not match"),
    }

    roll_blockchain(&mut router, 1).unwrap();

    // Assert No return shown
    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();
    assert_eq!(balances.0, 0);

    // MINT & DEPOSIT LP TOKEN & REWARD TOKEN
    mint_deposit_snip20(
        &mut router,
        &lp_token_contract,
        &owner_addr,
        Uint128::new(100000000),
        &owner_addr,
    );
    mint_deposit_snip20(
        &mut router,
        &reward_contract,
        &staking_contract.address,
        Uint128::new(100000000),
        &owner_addr,
    );
    // STAKE LP TOKEN
    send_snip20_to_stake(
        &mut router,
        &lp_token_contract,
        &staking_contract,
        Uint128::new(1000u128),
        &owner_addr,
        &owner_addr,
    )
    .unwrap();

    // Assert zero for the same time
    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();
    assert_eq!(balances.0, 1);
    assert_eq!(balances.1, Uint128::zero());

    // This is 5000 seconds
    roll_blockchain(&mut router, 1000).unwrap();

    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();
    assert_eq!(balances.0, 1);
    assert_eq!(balances.1, Uint128::new(17361));

    let msg = ExecuteMsg::ClaimRewards {};
    router
        .execute_contract(
            owner_addr.to_owned(),
            &staking_contract.clone(),
            &msg,
            &[], //
        )
        .unwrap();

    // Assert claim_rewards to set claimable_reward to zero (already paid)
    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();

    //Balance should now be 0 after claim
    assert_eq!(balances.0, 1);
    assert_eq!(balances.1, Uint128::zero());

    // ADD SECOND REWARD TOKEN
    let reward_contract_b =
        generate_snip20_contract(&mut router, "RWD".to_string(), "RWD".to_string(), 18).unwrap();
    let set_reward_msg = ExecuteMsg::SetRewardToken {
        reward_token: TokenType::CustomToken {
            contract_addr: reward_contract_b.address.to_owned(),
            token_code_hash: reward_contract_b.code_hash.to_owned(),
        },
        daily_reward_amount: Uint128::new(600000u128),
        valid_to: Uint128::new(3747905010000u128),
    };

    let _ = router
        .execute_contract(
            owner_addr.to_owned(),
            &staking_contract,
            &set_reward_msg,
            &[],
        )
        .unwrap();
    mint_deposit_snip20(
        &mut router,
        &reward_contract_b,
        &staking_contract.address,
        Uint128::new(100000),
        &owner_addr,
    );
    // This will move time forwards 2500
    roll_blockchain(&mut router, 500).unwrap();

    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();

    // There will only be one token as the user has not done an action yet
    assert_eq!(balances.0, 1);
    assert_eq!(balances.1, Uint128::new(8680u128));

    let msg = ExecuteMsg::ClaimRewards {};
    router
        .execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            &staking_contract.clone(),
            &msg,
            &[], //
        )
        .unwrap();

    // Assert 2 Reward Token + Claimable Reward
    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();

    assert_eq!(balances.0, 2);
    assert_eq!(balances.1, Uint128::new(0u128));
    assert_eq!(balances.2, Uint128::new(0u128));

    // Assert New Staker A
    mint_deposit_snip20(
        &mut router,
        &lp_token_contract,
        &staker_a_addr,
        Uint128::new(10000u128),
        &owner_addr,
    );
    let _ = send_snip20_to_stake(
        &mut router,
        &lp_token_contract,
        &staking_contract,
        Uint128::new(1000u128),
        &staker_a_addr,
        &staker_a_addr,
    )
    .unwrap();
    //Increment time by 5000
    roll_blockchain(&mut router, 1000).unwrap();

    // Check owner balance
    {
        // Query Balance
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            OWNER_PUB_KEY,
            OWNER_PUB_KEY,
            get_current_block_time(&router),
        )
        .unwrap();

        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(8680u128));
        assert_eq!(balances.2, Uint128::new(17361u128));
    }

    // Check staker A balance
    {
        // Query Balance
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            PUB_KEY_STAKER_A,
            PUB_KEY_STAKER_A,
            get_current_block_time(&router),
        )
        .unwrap();

        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(8680u128));
        assert_eq!(balances.2, Uint128::new(17361u128));
    }

    // Increment time by 5000
    roll_blockchain(&mut router, 1000).unwrap();

    let unstake_msg = ExecuteMsg::Unstake {
        amount: Uint128::new(500u128),
        remove_liquidity: Some(false),
    };

    let _ = router
        .execute_contract(owner_addr.to_owned(), &staking_contract, &unstake_msg, &[])
        .unwrap();
    // Assert owner balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            OWNER_PUB_KEY,
            OWNER_PUB_KEY,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(0u128));
        assert_eq!(balances.2, Uint128::new(0u128));
    }

    // Assert staker A balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            PUB_KEY_STAKER_A,
            PUB_KEY_STAKER_A,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(17361u128));
        assert_eq!(balances.2, Uint128::new(34722u128));
    }

    // Increment time by 5000
    roll_blockchain(&mut router, 1000).unwrap();

    //Assert owner balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            OWNER_PUB_KEY,
            OWNER_PUB_KEY,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(5787u128));
        assert_eq!(balances.2, Uint128::new(11574u128));
    }

    // Assert staker A balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            PUB_KEY_STAKER_A,
            PUB_KEY_STAKER_A,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(28935u128));
        assert_eq!(balances.2, Uint128::new(57870u128));
    }

    // Assert Unstake the whole amount
    let _ = router
        .execute_contract(owner_addr.to_owned(), &staking_contract, &unstake_msg, &[])
        .unwrap();
    let balances = query_claimable_reward(
        &router,
        &staking_contract,
        OWNER_PUB_KEY,
        OWNER_PUB_KEY,
        get_current_block_time(&router),
    )
    .unwrap();
    assert_eq!(balances.0, 2);
    assert_eq!(balances.1, Uint128::new(0u128));
    assert_eq!(balances.2, Uint128::new(0u128));

    //OWNER STAKE ON BEHALF OF STAKER_A
    let _ = send_snip20_to_proxy_stake(
        &mut router,
        &lp_token_contract,
        &staking_contract,
        Uint128::new(1000u128),
        &staker_a_addr,
        &owner_addr,
    )
    .unwrap();

    // Increment time by 5000
    roll_blockchain(&mut router, 1000).unwrap();

    //Assert owner balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            OWNER_PUB_KEY,
            OWNER_PUB_KEY,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(0u128));
        assert_eq!(balances.2, Uint128::new(0u128));
    }

    // Assert staker A balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            PUB_KEY_STAKER_A,
            PUB_KEY_STAKER_A,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(46296u128));
        assert_eq!(balances.2, Uint128::new(92592u128));
    }

    // OWNER cannot unstake the funds he put in
    assert!(
        router
            .execute_contract(
                owner_addr.to_owned(),
                &staking_contract,
                &ExecuteMsg::Unstake {
                    amount: Uint128::new(500u128),
                    remove_liquidity: Some(false),
                },
                &[]
            )
            .is_err()
            == true
    );

    // STAKER_A cannot unstake more then he put in
    assert!(
        router
            .execute_contract(
                staker_a_addr.to_owned(),
                &staking_contract,
                &ExecuteMsg::Unstake {
                    amount: Uint128::new(1500u128),
                    remove_liquidity: Some(false),
                },
                &[]
            )
            .is_err()
            == true
    );

    // SET VIEWKEY
    let view_key = "VIEWING_KEY";
    set_viewing_key(&mut router, &lp_token_contract, view_key, &owner_addr).unwrap();
    let lp_token_balance = get_snip20_balance(
        &mut router,
        &lp_token_contract,
        &owner_addr.to_string(),
        view_key,
    );
    // OWNER can unstake the funds he put in
    assert!(
        router
            .execute_contract(
                owner_addr.to_owned(),
                &staking_contract,
                &ExecuteMsg::ProxyUnstake {
                    amount: Uint128::new(500u128),
                    for_addr: staker_a_addr.to_string(),
                },
                &[]
            )
            .is_ok()
            == true
    );
    // Make sure owner gets funds back
    assert_eq!(
        get_snip20_balance(
            &mut router,
            &lp_token_contract,
            &owner_addr.to_string(),
            view_key
        ),
        lp_token_balance + Uint128::new(500)
    );

    // STAKE LP TOKEN
    send_snip20_to_stake(
        &mut router,
        &lp_token_contract,
        &staking_contract,
        Uint128::new(500u128),
        &owner_addr,
        &owner_addr,
    )
    .unwrap();

    // Increment time by 5000
    roll_blockchain(&mut router, 1000).unwrap();

    //Assert owner balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            OWNER_PUB_KEY,
            OWNER_PUB_KEY,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(4340u128));
        assert_eq!(balances.2, Uint128::new(8680u128));
    }

    // Assert staker A balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            PUB_KEY_STAKER_A,
            PUB_KEY_STAKER_A,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(59316u128));
        assert_eq!(balances.2, Uint128::new(118633u128));
    }

    // CHANGE EXISTING REWARD TOKEN
    let set_reward_msg = ExecuteMsg::SetRewardToken {
        reward_token: TokenType::CustomToken {
            contract_addr: reward_contract_b.address.to_owned(),
            token_code_hash: reward_contract_b.code_hash.to_owned(),
        },
        daily_reward_amount: Uint128::new(500000u128),
        valid_to: Uint128::new(3747905010000u128),
    };

    let _ = router
        .execute_contract(
            owner_addr.to_owned(),
            &staking_contract,
            &set_reward_msg,
            &[],
        )
        .unwrap();

    // Increment time by 5000
    roll_blockchain(&mut router, 1000).unwrap();

    //Assert owner balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            OWNER_PUB_KEY,
            OWNER_PUB_KEY,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(8680u128));
        assert_eq!(balances.2, Uint128::new(15914u128));
    }

    // Assert staker A balance
    {
        let balances = query_claimable_reward(
            &router,
            &staking_contract,
            PUB_KEY_STAKER_A,
            PUB_KEY_STAKER_A,
            get_current_block_time(&router),
        )
        .unwrap();
        assert_eq!(balances.0, 2);
        assert_eq!(balances.1, Uint128::new(72337u128));
        assert_eq!(balances.2, Uint128::new(140335u128));
    }
}

pub mod staking_help_query {
    use cosmwasm_std::{to_binary, ContractInfo, StdError, StdResult, Uint128};
    use multi_test::{
        help_lib::integration_help_lib::mk_create_permit_data, util_addr::util_blockchain::CHAIN_ID,
    };
    use secret_multi_test::App;
    use shadeswap_shared::staking::{AuthQuery, QueryMsg, QueryResponse};
    use shadeswap_shared::utils::testing::TestingExt;

    pub fn query_claimable_reward(
        router: &App,
        staking_contract: &ContractInfo,
        pub_key: &str,
        signature: &str,
        time: Uint128,
    ) -> (StdResult<(u8, Uint128, Uint128)>) {
        let permit = mk_create_permit_data(pub_key, signature, CHAIN_ID).unwrap();
        let query: QueryResponse = router.query_test(
            staking_contract.to_owned(),
            to_binary(&QueryMsg::WithPermit {
                permit,
                query: AuthQuery::GetClaimReward { time },
            })?,
        )?;
        match query {
            QueryResponse::GetClaimReward { claimable_rewards } => {
                let claimable_rewards_len = claimable_rewards.len();
                if claimable_rewards_len == 0 {
                    return Ok((0, Uint128::zero(), Uint128::zero()));
                } else if claimable_rewards_len == 1 {
                    return Ok((1, claimable_rewards[0].amount, Uint128::zero()));
                } else if claimable_rewards_len == 2 {
                    return Ok((2, claimable_rewards[0].amount, claimable_rewards[1].amount));
                } else {
                    Err(StdError::generic_err("Too many claimable rewards"))
                }
            }
            _ => Err(StdError::generic_err("No matching result")),
        }
    }
}
