pub mod snip20_lib {
    use crate::utils::{init_snip20_cli, InitConfig, GAS};
    use cosmwasm_std::Addr;
    use secretcli::{
        cli_types::NetContract,
        secretcli::{handle, query, Report},
    };
    use serde_json::Result;
    use snip20_reference_impl::msg::QueryAnswer;
    use std::io;

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

    use cosmwasm_std::{Binary, Uint128};
    use secretcli::{
        cli_types::{NetContract},
        secretcli::{handle, store_and_return_contract, Report, init},
    };
    use shadeswap_shared::{
        amm_pair::AMMSettings,
        c_std::{to_binary, Addr},
        core::{ContractInstantiationInfo, Fee},
        msg::factory::InitMsg as FactoryInitMsg,
        contract_interfaces::admin::InstantiateMsg as AdminInstantiateMsg,
        Contract, staking::{InvokeMsg}, query_auth::PermitData, admin::RegistryAction,
    };
    use query_authentication::permit::Permit;

    use crate::utils::{init_contract_factory, GAS, STORE_GAS, ADMIN_FILE, generate_label, print_header};

    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";

    pub fn create_admin_contract(
        account_name: &str,
        backend: &str,
        address: &str,
        super_address: &str,
        reports: &mut Vec<Report>,
    ) -> io::Result<NetContract> {     
        type TestPermit = Permit<PermitData>;
        //secretd tx sign-doc file --from a  
    
        print_header("\n\tInitializing Admin Contract");    
        let admin_msg = AdminInstantiateMsg {
            super_admin: Some(address.to_string()),
        };
    
        let admin_contract = init(
            &admin_msg,
            &ADMIN_FILE,
            &*generate_label(8),
            account_name,
            Some(STORE_GAS),
            Some(GAS),
            Some(backend),
            reports,
        )?;
    
        let admin_register_msg = RegistryAction::RegisterAdmin {
            user: super_address.to_string(),
        };
    
        handle(
            &admin_register_msg,
            &admin_contract,
            account_name,
            Some(GAS),
            Some(backend),
            Some("1000000000000uscrt"),
            reports,
            None,
        )?;

        Ok(admin_contract)
    }

    
    pub fn create_factory_contract(
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>,
        api_key: &str,
        seed: &str,
        lp_fee_nom: u8,
        lp_fee_denom: u16,
        shade_dao_fee_nom: u8,
        shade_dao_fee_denom: u16,
        shade_dao_address: &str,
        shade_dao_code_hash: &str,
        admin_contract: &str,
        admin_contract_code_hash: &str,
        auth_addr: &str,
        auth_code_hash: &str,
    ) -> io::Result<NetContract> {
        println!("Creating New Factory");
        let lp_token = store_and_return_contract(
            &LPTOKEN20_FILE,
            &account_name,
            Some(STORE_GAS),
            Some(backend),
        )?;

        let mut auth_contract: Option<Contract> = None;
        if auth_addr != "" {
            auth_contract = Some(Contract {
                address: Addr::unchecked(auth_addr),
                code_hash: auth_code_hash.to_string(),
            })
        }

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
                    code_hash: shade_dao_code_hash.to_string(),
                },
            },
            lp_token_contract: ContractInstantiationInfo {
                code_hash: lp_token.code_hash.to_string().clone(),
                id: lp_token.id.clone().parse::<u64>().unwrap(),
            },
            prng_seed: to_binary(seed).unwrap(),
            api_key: api_key.to_string(),
            authenticator: auth_contract,
            admin_auth: Contract {
                address: Addr::unchecked(admin_contract.to_string()),
                code_hash: admin_contract_code_hash.to_string(),
            },
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

        let msg_binary: Option<Binary> = match msg {
            Some(mg) => Some(to_binary(&mg).unwrap()),
            None => None,
        };

        let rec_code_hash = match recipient_code_hash {
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
            recipient: Addr::unchecked(token_addr.to_string()),
            recipient_code_hash: rec_code_hash,
            amount: snip_20_amount,
            msg: msg_binary,
            memo: None,
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

    pub fn send_snip_with_msg_staking(
        account_name: &str,
        backend: &str,
        sender: &str,
        token_addr: &str,
        snip_20_amount: Uint128,
        recipient: &str,
        recipient_code_hash: Option<String>,       
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        println!(
            "Send to SNIP20 - token {} - amount {} - recipient {}",
            token_addr.to_string(),
            snip_20_amount.to_string(),
            recipient.to_string()
        );

        let msg = to_binary(&InvokeMsg::Stake { from: recipient.to_string()}).unwrap();

        let rec_code_hash = match recipient_code_hash {
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
            recipient: Addr::unchecked(token_addr.to_string()),
            recipient_code_hash: rec_code_hash,
            amount: snip_20_amount,
            msg: Some(msg),
            memo: None,
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
        admin_code_hash: String,
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>,
        admin: &str,
    ) -> io::Result<NetContract> {
        let router_msg = RouterInitMsg {
            prng_seed: to_binary(&"".to_string()).unwrap(),
            entropy: to_binary(&"".to_string()).unwrap(),
            admin_auth: Contract {
                address: Addr::unchecked(admin.to_string()),
                code_hash: admin_code_hash.to_string(),
            },
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
        amm_pair::AMMPair,
        c_std::{to_binary, Addr},
        core::{ContractInstantiationInfo, TokenPair, TokenPairAmount, TokenType},
        msg::{
            amm_pair::{
                ExecuteMsg as AMMPairHandlMsg, QueryMsg as AMMPairQueryMsg,
                QueryMsgResponse as AMMPairQueryMsgResponse,
            },
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

    use super::factory_lib::increase_allowance;

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
            store_and_return_contract(STAKING_FILE, account_name, Some(STORE_GAS), Some(backend))?;
        Ok(stored_amm_pairs)
    }

    
    pub fn add_amm_pairs(
        factory_addr: String,
        factory_code_hash: String,
        backend: &str,
        account_name: &str,
        token_0_address: String,
        token_0_code_hash: String,
        token_1_address: String,
        token_1_code_hash: String,
        entropy: &str,
        reward_contract_address: Option<String>,
        reward_contract_code_hash: Option<String>,
        reward_amount: Option<u128>,
        valid_to: Option<u128>,
        lp_token_decimals: u8,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        let factory_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: factory_addr.clone(),
            code_hash: factory_code_hash,
        };

        let pairs: Option<TokenPair>;
        if &token_0_address == "" {
            pairs = Some(TokenPair(
                TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_1_address.clone()),
                    token_code_hash: token_1_code_hash.clone(),
                },
            ));
        } else {
            pairs = Some(TokenPair(
                TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_0_address.clone()),
                    token_code_hash: token_0_code_hash.clone(),
                },
                TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_1_address.clone()),
                    token_code_hash: token_1_code_hash.clone(),
                },
            ));
        }

        let staking_contract = store_staking_contract(&account_name, &backend)?;
        let staking_contract_init: Option<StakingContractInit> = match reward_contract_address {
            Some(msg) => Some(StakingContractInit {
                contract_info: ContractInstantiationInfo {
                    code_hash: staking_contract.code_hash.to_string(),
                    id: staking_contract.id.clone().parse::<u64>().unwrap(),
                },
                daily_reward_amount: Uint128::from(reward_amount.unwrap()),
                reward_token: TokenType::CustomToken {
                    contract_addr: Addr::unchecked(msg.clone()),
                    token_code_hash: reward_contract_code_hash.unwrap().to_string(),
                },
                valid_to: Uint128::new(valid_to.unwrap())
            }),
            None => None,
        };

        handle(
            &FactoryExecuteMsg::CreateAMMPair {
                pair: pairs.unwrap().clone(),
                entropy: to_binary(&entropy).unwrap(),
                staking_contract: staking_contract_init,
                lp_token_decimals: lp_token_decimals
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

    pub fn list_pair_from_factory(
        factory_addr: String,
        start: u64,
        limit: u8,
    ) -> io::Result<Vec<AMMPair>> {
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
            return Ok(amm_pairs);
        }
        return Ok(Vec::new());
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
                reward_token: TokenType::CustomToken { contract_addr: Addr::unchecked(token_addr.to_string()), token_code_hash: token_code_hash.to_string() } ,
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

    pub fn get_staking_contract(amm_pair_address: &str) -> io::Result<Option<Contract>> {
        let staking_contract_msg = AMMPairQueryMsg::GetConfig {};
        let staking_contract_query: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_address.to_string(),
                code_hash: "".to_string(),
            },
            staking_contract_msg,
            None,
        )?;
        if let AMMPairQueryMsgResponse::GetConfig { staking_contract, factory_contract: _, lp_token: _, pair: _, custom_fee: _ } =
            staking_contract_query
        {
            return Ok(staking_contract);
        }
        return Ok(None);
    }

    pub fn get_lp_liquidity(amm_pair_address: &str) -> io::Result<Option<Uint128>>{
        let lp_token_info_msg = AMMPairQueryMsg::GetPairInfo {};
        let lp_token_info_query_unstake: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_address.to_string(),
                code_hash: "".to_string(),
            },
            lp_token_info_msg,
            None,
        )?;
        if let AMMPairQueryMsgResponse::GetPairInfo {
            liquidity_token,
            factory: _,
            pair: _,
            amount_0: _,
            amount_1: _,
            total_liquidity,
            contract_version: _,
            fee_info: _,
        } = lp_token_info_query_unstake{
            return Ok(Some(total_liquidity))
        }

        return Ok(Some(Uint128::zero()));
    }

    pub fn get_lp_contract(amm_pair_address: &str) -> io::Result<Option<Contract>> {
        let staking_contract_msg = AMMPairQueryMsg::GetConfig {};
        let staking_contract_query: AMMPairQueryMsgResponse = query(
            &NetContract {
                label: "".to_string(),
                id: "".to_string(),
                address: amm_pair_address.to_string(),
                code_hash: "".to_string(),
            },
            staking_contract_msg,
            None,
        )?;
        if let AMMPairQueryMsgResponse::GetConfig { staking_contract, factory_contract: _, lp_token, pair: _, custom_fee: _ } =
            staking_contract_query
        {
            return Ok(Some(lp_token));
        }
        return Ok(None);
    }

    pub fn add_liquidity(
        account_name: &str,
        backend: &str,
        pair_addr: String,
        token_0_addr: String,
        token_0_code_hash: String,
        token_1_addr: String,
        token_1_code_hash: String,
        amount_0: Uint128,
        amount_1: Uint128,
        staking_opt: bool,
        exp_return: &str,
        reports: &mut Vec<Report>,
    ) -> io::Result<()> {
        let pair_contract = NetContract {
            label: "".to_string(),
            id: "".to_string(),
            address: pair_addr.clone(),
            code_hash: "".to_string(),
        };

        let pair: Option<TokenPair>;
        let mut native_amount: Option<String> = None;
        if token_0_addr == "" || token_1_addr == "" {
            if token_0_addr == "" {
                let mut amo = amount_0.to_owned().to_string();
                let denom = "uscrt".to_string();
                amo.push_str(&denom);
                native_amount = Some(amo.to_string());
                pair = Some(TokenPair(
                    TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    TokenType::CustomToken {
                        contract_addr: Addr::unchecked(token_1_addr.clone()),
                        token_code_hash: token_1_code_hash.clone(),
                    },
                ));

                // increase allowance
                increase_allowance(
                    pair_addr.to_owned(),
                    amount_1,
                    token_1_addr,
                    account_name,
                    backend,
                    reports,
                )
                .unwrap();
            } else {
                let mut amo = amount_1.to_owned().to_string();
                let denom = "uscrt".to_string();
                amo.push_str(&denom);
                native_amount = Some(amo.to_string());
                pair = Some(TokenPair(
                    TokenType::CustomToken {
                        contract_addr: Addr::unchecked(token_0_addr.clone()),
                        token_code_hash: token_0_code_hash.clone(),
                    },
                    TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                ));

                // increase allowance
                increase_allowance(
                    pair_addr.to_owned(),
                    amount_0,
                    token_0_addr,
                    account_name,
                    backend,
                    reports,
                )
                .unwrap();
            }
        } else {
            // increase allowance
            increase_allowance(
                pair_addr.to_owned(),
                amount_0,
                token_0_addr.to_owned(),
                account_name,
                backend,
                reports,
            )
            .unwrap();
            increase_allowance(
                pair_addr.to_owned(),
                amount_1,
                token_1_addr.to_owned(),
                account_name,
                backend,
                reports,
            )
            .unwrap();

            pair = Some(TokenPair(
                TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_0_addr.clone()),
                    token_code_hash: token_0_code_hash.clone(),
                },
                TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_1_addr.clone()),
                    token_code_hash: token_1_code_hash.clone(),
                },
            ));
        }

        let mut expected_return: Option<Uint128> = None;
        if exp_return != "" {
            expected_return = Some(Uint128::new(exp_return.parse::<u128>().unwrap()));
        }

        let mut staking: Option<bool> = None;
        if staking_opt == true {
            staking = Some(true);
        }

        handle(
            &AMMPairHandlMsg::AddLiquidityToAMMContract {
                deposit: TokenPairAmount {
                    pair: pair.unwrap(),
                    amount_0: amount_0,
                    amount_1: amount_1,
                },
                expected_return: expected_return,
                staking: staking,
            },
            &pair_contract,
            account_name,
            Some(GAS),
            Some(backend),
            native_amount.as_ref().map(String::as_ref),
            reports,
            None,
        )
        .unwrap();

        Ok(())
    }
}
