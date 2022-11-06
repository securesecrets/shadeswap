pub mod snip20_lib {
    use std::io;
    use serde_json::Result;
    use secretcli::{
        cli_types::NetContract,
        secretcli::{handle, query, Report},
    };
    use snip20_reference_impl::msg::QueryAnswer;
    use crate::utils::{init_snip20_cli, InitConfig, GAS};
    use cosmwasm_std::Addr;

    pub fn create_new_snip_20(
        account_name: &str,
        backend: &str,
        name: &str,
        symbol: &str,
        decimal: u8,
        viewing_key: &str,
        reports: &mut Vec<Report>,
        enable_burn: bool,
        enable_mint: bool,
        enable_deposit: bool,
        enable_redeem: bool,
        public_total_sypply: bool,
    ) -> io::Result<NetContract> {
        println!(
            "Creating SNIP20 token - Name: {}, Symbol: {}, Decimals: {}",
            name, symbol, decimal
        );
        let snip20 = init_snip20_contract(
            &name.trim(),
            &symbol.trim(),
            reports,
            decimal,
            account_name,
            backend,
            enable_burn,
            enable_mint,
            enable_deposit,
            enable_redeem,
            public_total_sypply,
        )?;

        let contract = NetContract {
            label: snip20.label.to_string(),
            id: snip20.id.clone().to_string(),
            code_hash: snip20.code_hash.clone(),
            address: snip20.address.clone().to_string(),
        };

        set_viewing_key(
            viewing_key,
            &contract.clone(),
            reports,
            account_name,
            backend,
        )?;
        Ok(contract)
    }

    pub fn init_snip20_contract(
        symbol: &str,
        name: &str,
        reports: &mut Vec<Report>,
        decimal: u8,
        account_name: &str,
        keyring_backend: &str,
        enable_burn: bool,
        enable_mint: bool,
        enable_deposit: bool,
        enable_redeem: bool,
        public_total_sypply: bool,
    ) -> Result<NetContract> {
        let config = InitConfig {
            enable_burn: Some(enable_burn),
            enable_mint: Some(enable_mint),
            enable_deposit: Some(enable_deposit),
            enable_redeem: Some(enable_redeem),
            public_total_supply: Some(public_total_sypply),
        };

        let s_contract = init_snip20_cli(
            name.to_string(),
            symbol.to_string(),
            decimal,
            Some(config),
            reports,
            &account_name,
            None,
            &keyring_backend,
        )?;

        println!("Contract address - {}", s_contract.1.address.clone());
        println!("Code hash - {}", s_contract.1.code_hash.clone());
        println!("Code Id - {}", s_contract.1.id);

        Ok(s_contract.1)
    }

    pub fn set_viewing_key(
        viewing_key: &str,
        net_contract: &NetContract,
        reports: &mut Vec<Report>,
        account_name: &str,
        backend: &str,
    ) -> io::Result<()> {
        let msg = snip20_reference_impl::msg::ExecuteMsg::SetViewingKey {
            key: String::from(viewing_key),
            padding: None,
        };

        handle(
            &msg,
            &net_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )?;
        Ok(())
    }

    pub fn balance_snip20_query(
        snip20_addr: String,
        spender: String,
        key: String,
    ) -> io::Result<()> {
        let msg = &snip20_reference_impl::msg::QueryMsg::Balance {
            address: Addr::unchecked(spender.clone()),
            key: key.clone(),
        };

        let snip20_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: snip20_addr.clone(),
            code_hash: "".to_string(),
        };
        let snip_query: QueryAnswer = query(&snip20_contract, msg, None)?;
        if let QueryAnswer::Balance { amount } = snip_query {
            println!(
                "Balance Snip20 {} - address {} - amount {}",
                snip20_addr.clone(),
                spender.clone(),
                amount
            );
        }

        Ok(())
    }
}

pub mod factory_lib {
    use std::io;

    use cosmwasm_std::{Uint128, Binary};
    use secretcli::{
        cli_types::NetContract,
        secretcli::{handle, store_and_return_contract, Report},
    };
    use shadeswap_shared::{
        amm_pair::AMMSettings,
        c_std::{to_binary, Addr},
        core::{ContractInstantiationInfo, Fee},
        msg::factory::InitMsg as FactoryInitMsg,
        Contract,
    };

    use crate::utils::{init_contract_factory, API_KEY, GAS, STORE_GAS};

    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";

    pub fn create_factory_contract(
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>,
        admin_contract: &str,
        api_key:&str,
        seed: &str,
        shade_dao_address: &str,
        lp_fee_nom: u8,
        lp_fee_denom: u16,
        shade_dao_fee_nom: u8,
        shade_dao_fee_denom: u16,  
        auth_addr: Option<String>,       
    ) -> io::Result<NetContract> {
        println!("Creating New Factory");
        let lp_token = store_and_return_contract(
            &LPTOKEN20_FILE,
            &account_name,
            Some(STORE_GAS),
            Some(backend),
        )?;

        let authenticator = match auth_addr {
            Some(addr) => Some(Contract{address: Addr::unchecked(addr), code_hash: "".to_string()}),
            None => None,
        };

        let pair_contract = store_and_return_contract(
            &AMM_PAIR_FILE,
            &account_name,
            Some(STORE_GAS),
            Some(backend),
        )?;

        let init_msg = FactoryInitMsg {
            pair_contract: ContractInstantiationInfo {
                code_hash: pair_contract.code_hash.to_string().clone(),
                id: pair_contract.id.clone().parse::<u64>().unwrap(),
            },
            amm_settings: AMMSettings {
                shade_dao_fee: Fee::new(shade_dao_fee_nom, shade_dao_fee_denom),
                lp_fee: Fee::new(lp_fee_nom, lp_fee_denom),
                shade_dao_address: Contract {
                    address: Addr::unchecked(shade_dao_address.to_string()),
                    code_hash: "".to_string(),
                },
            },
            lp_token_contract: ContractInstantiationInfo {
                code_hash: lp_token.code_hash.to_string().clone(),
                id: lp_token.id.clone().parse::<u64>().unwrap(),
            },
            prng_seed: to_binary(seed).unwrap(),
            api_key: API_KEY.to_string(),
            authenticator: authenticator,
            admin_auth: Contract{address: Addr::unchecked(admin_contract.to_string()), code_hash: "".to_string()},
        };

        let factory_contract =
            init_contract_factory(&account_name, &backend, &FACTORY_FILE, &init_msg, reports)?;

        Ok(factory_contract)
    }

    pub fn deposit_snip20(
        account_name: &str,
        backend: &str,
        token_addr: &str,
        amount: &str,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Deposit to SNIP20 - token {} - amount {}",
            token_addr.to_string(),
            amount
        );
        let net_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: token_addr.to_string(),
            code_hash: "".to_string(),
        };

        let msg = snip20_reference_impl::msg::ExecuteMsg::Deposit { padding: None };
        handle(
            &msg,
            &net_contract,
            account_name,
            Some(GAS),
            Some(backend),
            Some(amount),
            reports,
            None,
        )?;
        Ok(())
    }

    pub fn mint_snip20(
        account_name: &str,
        backend: &str,
        recipient: String,
        amount: Uint128,
        amount_uscrt: &str,
        reports: &mut Vec<Report>,
        snip20_addr: String,
    ) -> io::Result<()> {
        println!(
            "Minting SNIP20 {} - recipient {} - amount {} - amount scrt {}",
            snip20_addr.clone(),
            recipient.clone(),
            amount,
            amount_uscrt.clone()
        );
        let net_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: snip20_addr.clone(),
            code_hash: "".to_string(),
        };
        let msg = snip20_reference_impl::msg::ExecuteMsg::Mint {
            padding: None,
            recipient: Addr::unchecked(recipient.clone()),
            amount: amount,
            memo: None,
        };
        handle(
            &msg,
            &net_contract,
            account_name,
            Some(GAS),
            Some(backend),
            Some(amount_uscrt),
            reports,
            None,
        )?;
        Ok(())
    }

    pub fn send_snip_with_msg(
        account_name: &str,
        backend: &str,
        token_addr: &str,
        snip_20_amount: Uint128,
        recipient: &str,
        recipient_code_hash: Option<String>,
        msg: Option<String>,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Send to SNIP20 - token {} - amount {} - recipient {}",
            token_addr.to_string(),
            snip_20_amount.to_string(),
            recipient.to_string()
        );

        let msg_binary: Option<Binary> = match msg{
            Some(mg) => Some(to_binary(&mg).unwrap()),
            None => None,
        };

        let rec_code_hash = match recipient_code_hash{
            Some(hash) => Some(hash),
            None => None,
        };

        let net_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: token_addr.to_string(),
            code_hash: "".to_string(),
        };

        let msg = snip20_reference_impl::msg::ExecuteMsg::Send { 
            recipient: Addr::unchecked(recipient.to_string()), 
            recipient_code_hash: rec_code_hash, 
            amount: snip_20_amount, 
            msg: msg_binary, 
            memo: None, 
            padding: None 
        };

        handle(
            &msg,
            &net_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )?;
        Ok(())
    }


    pub fn increase_allowance(
        spender: String,
        amount: Uint128,
        snip20_addr: String,
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Increase Allowance SNIP20 {} - spender {} - amount {}",
            snip20_addr.clone(),
            spender.clone(),
            amount
        );
        let net_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: snip20_addr.clone(),
            code_hash: "".to_string(),
        };
        handle(
            &snip20_reference_impl::msg::ExecuteMsg::IncreaseAllowance {
                spender: Addr::unchecked(spender.clone()),
                amount: amount,
                expiration: None,
                padding: None,
            },
            &net_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )
        .unwrap();
        Ok(())
    }
}

pub mod router_lib {
    use std::io;

    use secretcli::{
        cli_types::NetContract,
        secretcli::{handle, init, Report},
    };
    use shadeswap_shared::utils::asset::Contract;
    use shadeswap_shared::{
        c_std::{to_binary, Addr},
        msg::router::{ExecuteMsg as RouterExecuteMsg, InitMsg as RouterInitMsg},
    };

    use crate::utils::{generate_label, GAS, STORE_GAS};

    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
    pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";

    pub fn create_router_contract(
        code_hash: String,
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>,
        admin: &str
    ) -> io::Result<NetContract> {
        println!(
            "Creating New Router Contract with Pair Code Hash {}",
            code_hash.clone()
        );
        let router_msg = RouterInitMsg {
            prng_seed: to_binary(&"".to_string()).unwrap(),
            entropy: to_binary(&"".to_string()).unwrap(),
            admin_auth: Contract { address: Addr::unchecked(admin.to_string()), code_hash: "".to_string()},
        };

        let router_contract = init(
            &router_msg,
            &ROUTER_FILE,
            &*generate_label(8),
            account_name,
            Some(STORE_GAS),
            Some(GAS),
            Some(backend),
            reports,
        )?;

        Ok(router_contract)
    }

    pub fn register_snip20_router(
        account_name: &str,
        backend: &str,
        snip20_address: String,
        snip20_code_hash: String,
        router_address: String,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Registering SNIP20 {} {} to the Router {}",
            snip20_address.clone(),
            snip20_code_hash.clone(),
            router_address.clone()
        );
        let net_contract = NetContract {
            address: router_address.clone(),
            label: "".to_string(),
            id: "".to_string(),
            code_hash: "".to_string(),
        };

        handle(
            &RouterExecuteMsg::RegisterSNIP20Token {
                token_addr: snip20_address.clone(),
                token_code_hash: snip20_code_hash.clone(),
            },
            &net_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )
        .unwrap();

        Ok(())
    }
}

pub mod amm_pair_lib {
    use cosmwasm_std::Uint128;
    use secretcli::{
        cli_types::{NetContract, StoredContract},
        secretcli::{handle, query, store_and_return_contract, Report},
    };

    use shadeswap_shared::{
        c_std::{to_binary, Addr},
        core::{ContractInstantiationInfo, TokenPair, TokenPairAmount, TokenType},
        msg::{
            amm_pair::ExecuteMsg as AMMPairHandlMsg,
            factory::{
                ExecuteMsg as FactoryExecuteMsg, QueryMsg as FactoryQueryMsg,
                QueryResponse as FactoryQueryResponse,
            },
            staking::{ExecuteMsg as StakingExecuteMsg, StakingContractInit},
        },
        Contract, Pagination,
    };
    use std::io;

    use crate::utils::{GAS, STORE_GAS};

    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
    pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";
    pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";

    pub fn store_amm_pair(account_name: &str, backend: &str) -> io::Result<StoredContract> {
        println!("Storing AMM Pair Contract");
        let stored_amm_pairs =
            store_and_return_contract(AMM_PAIR_FILE, account_name, Some(STORE_GAS), Some(backend))?;
        Ok(stored_amm_pairs)
    }

    pub fn store_staking_contract(account_name: &str, backend: &str) -> io::Result<StoredContract> {
        println!("Storing Staking Contract");
        let stored_amm_pairs =
            store_and_return_contract(AMM_PAIR_FILE, account_name, Some(STORE_GAS), Some(backend))?;
        Ok(stored_amm_pairs)
    }

    pub fn add_amm_pairs_with_staking(
        factory_addr: String,
        backend: &str,
        account_name: &str,
        token_0_address: String,
        token_1_address: String,
        token_code_hash: String,
        reward_contract_address: String,
        reward_contract_code_hash: String,
        reward_amount: Uint128,
        valid_to: Uint128,
        router_contract: Option<String>,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Creating New Pairs for factory {} - token_0 {} - token_1 {} - amount {} with staking",
            factory_addr.clone(),
            token_0_address.clone(),
            token_1_address.clone(),
            reward_amount
        );
        let factory_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: factory_addr.clone(),
            code_hash: "".to_string(),
        };

        let router_contr: Option<Contract> = match router_contract{
            Some(contract) => Some(Contract{ 
                address: Addr::unchecked(contract), 
                code_hash: "".to_string(),
            }),
            None => None
        };

        let pairs = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_0_address.clone()),
                token_code_hash: token_code_hash.clone(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_1_address.clone()),
                token_code_hash: token_code_hash.clone(),
            },
        );

        let staking_contract = store_staking_contract(&account_name, &backend)?;

        handle(
            &FactoryExecuteMsg::CreateAMMPair {
                pair: pairs.clone(),
                entropy: to_binary(&"".to_string()).unwrap(),
                staking_contract: Some(StakingContractInit {
                    contract_info: ContractInstantiationInfo {
                        code_hash: staking_contract.code_hash.to_string(),
                        id: staking_contract.id.clone().parse::<u64>().unwrap(),
                    },
                    daily_reward_amount: Uint128::from(reward_amount),
                    reward_token: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(reward_contract_address.clone()),
                        token_code_hash: reward_contract_code_hash.to_string(),
                    },
                    valid_to: valid_to,
                }),
                router_contract: router_contr,
            },
            &factory_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )
        .unwrap();

        Ok(())
    }

    pub fn add_amm_pairs_no_staking(
        factory_addr: String,
        backend: &str,
        account_name: &str,
        token_0_address: String,
        token_1_address: String,
        token_code_hash: String,
        router_contract: Option<String>,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Creating New Pairs for factory {} - token_0 {} - token_1 {} - no staking",
            factory_addr.clone(),
            token_0_address.clone(),
            token_1_address.clone()
        );
        let factory_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: factory_addr.clone(),
            code_hash: "".to_string(),
        };

        let pairs = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_0_address.clone()),
                token_code_hash: token_code_hash.clone(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_1_address.clone()),
                token_code_hash: token_code_hash.clone(),
            },
        );

        handle(
            &FactoryExecuteMsg::CreateAMMPair {
                pair: pairs.clone(),
                entropy: to_binary(&"".to_string()).unwrap(),
                staking_contract: None,
                router_contract: None,
            },
            &factory_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )
        .unwrap();
        Ok(())
    }

    pub fn list_pair_from_factory(factory_addr: String, start: u64, limit: u8) -> io::Result<()> {
        let factory_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: factory_addr.clone(),
            code_hash: "".to_string(),
        };
        let msg = FactoryQueryMsg::ListAMMPairs {
            pagination: Pagination {
                start: start,
                limit: limit,
            },
        };
        let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
        if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
            for i in 0..amm_pairs.len() {
                println!("{:?}", amm_pairs[i]);
            }
        }

        Ok(())
    }

    pub fn get_token_type(pairs: TokenPair) -> io::Result<(String, String)> {
        let token_0_address = match pairs.0 {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash: _,
            } => contract_addr.clone().to_string(),
            TokenType::NativeToken { denom: _ } => "".to_string(),
        };

        let token_1_address = match pairs.1 {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash: _,
            } => contract_addr.clone().to_string(),
            TokenType::NativeToken { denom: _ } => "".to_string(),
        };

        Ok((token_0_address, token_1_address))
    }

    pub fn set_reward_token(
        account_name: &str,
        backend: &str,
        staking_addr: &str,
        token_addr: &str,
        token_code_hash: &str,
        daily_reward_amount: Uint128,
        valid_to: Uint128,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        let staking_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: staking_addr.to_string(),
            code_hash: "".to_string(),
        };

        handle(
            &StakingExecuteMsg::SetRewardToken {
                reward_token: Contract {
                    address: Addr::unchecked(token_addr.to_string()),
                    code_hash: token_code_hash.to_string(),
                },
                daily_reward_amount: daily_reward_amount,
                valid_to: valid_to,
            },
            &staking_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )
        .unwrap();

        Ok(())
    }

    pub fn add_liquidity(
        account_name: &str,
        backend: &str,
        pair_addr: String,
        token_0_addr: String,
        token_1_addr: String,
        token_code_hash: String,
        amount_0: Uint128,
        amount_1: Uint128,
        staking_opt: bool,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        let pair_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: pair_addr.clone(),
            code_hash: "".to_string(),
        };

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_0_addr.clone()),
                token_code_hash: token_code_hash.clone(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_1_addr.clone()),
                token_code_hash: token_code_hash.clone(),
            },
        );

        let mut staking: Option<bool> = None;
        if staking_opt == true {
            staking = Some(true);
        }

        handle(
            &AMMPairHandlMsg::AddLiquidityToAMMContract {
                deposit: TokenPairAmount {
                    pair: pair.clone(),
                    amount_0: amount_0,
                    amount_1: amount_1,
                },
                expected_return: None,
                staking: staking,
            },
            &pair_contract,
            account_name,
            Some(GAS),
            Some(backend),
            None,
            reports,
            None,
        )
        .unwrap();

        Ok(())
    }
}
