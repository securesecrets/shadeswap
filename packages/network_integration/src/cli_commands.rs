
pub mod snip20_lib{
    use std::io;

    use secretcli::{secretcli::{Report, handle, query}, cli_types::NetContract};
    use snip20_reference_impl::msg::QueryAnswer;

    use crate::utils::{InitConfig, init_snip20_cli, GAS};
    use cosmwasm_std::Addr;
    pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
    
    pub fn create_new_snip_20(account_name: &str, backend: &str, name:&str, symbol:&str, decimal: u8, 
        viewing_key:&str, reports: &mut Vec<Report>,  enable_burn: bool, enable_mint: bool, enable_deposit: bool,
        enable_redeem: bool, public_total_sypply:bool) -> io::Result<NetContract>
    {       
        println!("Creating SNIP20 token - Name: {}, Symbol: {}, Decimals: {}", name, symbol, decimal);
        let snip20 = init_snip20_contract(&name.trim(), &symbol.trim(),
        reports, decimal, account_name, backend, enable_burn, enable_mint, enable_deposit,
        enable_redeem, public_total_sypply)?;
    
         let contract = NetContract{
            label: snip20.label.to_string(),
            id: snip20.id.clone().to_string(),
            code_hash: snip20.code_hash.clone(),
            address: snip20.address.clone().to_string()
        };
        
        set_viewing_key(viewing_key, &contract.clone(), reports,
            account_name, backend)?;
        Ok(contract)
    }
    
    pub fn init_snip20_contract(symbol: &str, name: &str, reports: &mut Vec<Report>, 
        decimal: u8, account_name: &str, keyring_backend: &str, enable_burn: bool, enable_mint: bool, enable_deposit: bool,
        enable_redeem: bool, public_total_sypply:bool) -> io::Result<NetContract>{
          
        let config = InitConfig{
            enable_burn: Some(enable_burn),
            enable_mint: Some(enable_mint),
            enable_deposit : Some(enable_deposit),
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
            Some(&SNIP20_FILE),
            &keyring_backend        
        )?;
    
        println!("Contract address - {}", s_contract.1.address.clone());
        println!("Code hash - {}", s_contract.1.code_hash.clone());
        println!("Code Id - {}", s_contract.1.id);
        
        Ok(s_contract.1)
    }

    fn set_viewing_key(
        viewing_key: &str, 
        net_contract: &NetContract, 
        reports: &mut Vec<Report>,
        account_name: &str,
        backend: &str) ->io::Result<()>{
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
        key: String
    ) -> io::Result<()>
    {
        let msg = &snip20_reference_impl::msg::QueryMsg::Balance { address: Addr::unchecked(spender.clone()), key: key.clone() };

        let snip20_contract = NetContract { label: "".to_string(), id: "".to_string(), address: snip20_addr.clone(), code_hash: "".to_string() };          
        let snip_query: QueryAnswer = query(&snip20_contract, msg, None)?;
        if let QueryAnswer::Balance {  amount } = snip_query {
            println!("Balance Snip20 {} - address {} - amount {}", snip20_addr.clone(), spender.clone(),amount);        
        }
    
        Ok(())
    }
}

pub mod factory_lib{
    use std::io;

    use cosmwasm_std::Uint128;
    use secretcli::{cli_types::NetContract, secretcli::{Report, store_and_return_contract, handle}};
    use shadeswap_shared::{
        amm_pair::{AMMSettings},
        core::{ContractInstantiationInfo, ContractLink, Fee},
        msg::{
            factory::{
                InitMsg as FactoryInitMsg,
            },
        },
        c_std::{Addr, to_binary},
    };

    use crate::utils::{init_contract_factory, STORE_GAS, GAS, API_KEY};
    
    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";

    pub fn create_factory_contract(account_name: &str, backend: &str, reports: &mut Vec<Report>) 
    -> io::Result<NetContract>
    {
        println!("Creating New Factory");
        let lp_token = 
            store_and_return_contract(&LPTOKEN20_FILE, 
                &account_name,
                Some(STORE_GAS),
                Some(backend)
            )?;

        let pair_contract = 
            store_and_return_contract(&AMM_PAIR_FILE, 
                &account_name,
                Some(STORE_GAS),
                Some(backend)
            )?;
        
        let init_msg = FactoryInitMsg{
            pair_contract: ContractInstantiationInfo{ 
                code_hash: pair_contract.code_hash.to_string().clone(), 
                id: pair_contract.id.clone().parse::<u64>().unwrap()
            },
            amm_settings: AMMSettings{
                shade_dao_fee: Fee::new(8, 100),
                lp_fee: Fee::new(2, 8),
                shade_dao_address:  ContractLink {
                    address: Addr::unchecked("".to_string()),
                    code_hash: "".to_string(),
                },
            },
            lp_token_contract: ContractInstantiationInfo{ 
                code_hash: lp_token.code_hash.to_string().clone(), 
                id: lp_token.id.clone().parse::<u64>().unwrap()
            },
            prng_seed:  to_binary(&"".to_string()).unwrap(),
            api_key: API_KEY.to_string(),
            authenticator: None,
        };
        
        let factory_contract = init_contract_factory(
            &account_name, 
            &backend,
            &FACTORY_FILE, 
            &init_msg, 
            reports
        )?;    
       
        Ok(factory_contract)
    }

    pub fn mint_snip20(
        account_name: &str,
        backend: &str,
        recipient: String,
        amount: Uint128, 
        amount_uscrt: &str, 
        reports: &mut Vec<Report>,
        snip20_addr: String
    ) -> io::Result<()>{
        println!("Minting SNIP20 {} - recipient {} - amount {} - amount scrt {}", snip20_addr.clone(), recipient.clone(),amount, amount_uscrt.clone());
        let net_contract = NetContract{
            label: "".to_string(),
            id: "".to_string(),
            address: snip20_addr.clone(),
            code_hash: "".to_string(),
        };
        let msg = snip20_reference_impl::msg::ExecuteMsg::Mint { padding: None, recipient: Addr::unchecked(recipient.clone()), amount: amount, memo: None };
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

    pub fn increase_allowance(
        spender: String,
        amount: Uint128,
        snip20_addr: String,
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>
    ) -> io::Result<()>
    {
        println!("Increase Allowance SNIP20 {} - spender {} - amount {}", snip20_addr.clone(), spender.clone(),amount);
        let net_contract = NetContract{
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

pub mod router_lib{
    use std::io;

    use secretcli::{cli_types::NetContract, secretcli::{Report, init, handle}};
    use shadeswap_shared::{
        msg::{
            router::{
                ExecuteMsg as RouterExecuteMsg, InitMsg as RouterInitMsg
            },
        },c_std::{Addr, to_binary},
    };

    use crate::utils::{STORE_GAS, GAS, generate_label};
    
    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
    pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";

    pub fn create_router_contract(code_hash: String,
        account_name: &str,
        backend: &str,
        reports: &mut Vec<Report>) -> io::Result<NetContract>
        {
            println!("Creating New Router Contract with Pair Code Hash {}", code_hash.clone());
            let router_msg = RouterInitMsg {
                prng_seed: to_binary(&"".to_string()).unwrap(),      
                entropy: to_binary(&"".to_string()).unwrap(),
                pair_contract_code_hash: code_hash,
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
            reports: &mut Vec<Report>
        ) -> io::Result<()>
        {
            println!("Registering SNIP20 {} {} to the Router {}",snip20_address.clone(),snip20_code_hash.clone(), router_address.clone());
            let net_contract = NetContract{
                address: router_address.clone(),
                label: "".to_string(),
                id: "".to_string(),
                code_hash: "".to_string(),
            };

            handle(
                &RouterExecuteMsg::RegisterSNIP20Token {
                    token_addr: Addr::unchecked(snip20_address.clone()),
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

pub mod amm_pair_lib{
    use cosmwasm_std::Uint128;
    use secretcli::{
        cli_types::{NetContract, StoredContract},
        secretcli::{handle, query, store_and_return_contract, Report},
    };

    use std::io;
    use shadeswap_shared::{
        core::{ContractInstantiationInfo, TokenType, TokenPair, TokenPairAmount},
        msg::{
            amm_pair::{
                ExecuteMsg as AMMPairHandlMsg
            },
            factory::{
                ExecuteMsg as FactoryExecuteMsg,
                QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse,
            },
            staking::StakingContractInit,
        },
        Pagination, c_std::{Addr, to_binary},
    };

    use crate::utils::{STORE_GAS, GAS};
    
    pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
    pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
    pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
    pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";
    pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";

    pub fn store_amm_pair(account_name: &str,
        backend: &str) -> io::Result<StoredContract> 
        {   
            println!("Storing AMM Pair Contract");
            let stored_amm_pairs = store_and_return_contract(AMM_PAIR_FILE, account_name, 
                Some(STORE_GAS), Some(backend))?;
            Ok(stored_amm_pairs) 
        }

    pub fn store_staking_contract(account_name: &str,
        backend: &str) -> io::Result<StoredContract> 
        {   
            println!("Storing Staking Contract");
            let stored_amm_pairs = store_and_return_contract(AMM_PAIR_FILE, account_name, 
                Some(STORE_GAS), Some(backend))?;
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
            reports: &mut Vec<Report>
        ) -> io::Result<()> {
            println!("Creating New Pairs for factory {} - token_0 {} - token_1 {} - amount {} with staking", factory_addr.clone(), token_0_address.clone(),token_1_address.clone(), reward_amount);
            let factory_contract = NetContract { label: "".to_string(), id: "".to_string(), address: factory_addr.clone(), code_hash: "".to_string() };

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
                        amount: Uint128::from(reward_amount),
                        reward_token: TokenType::CustomToken {
                            contract_addr: Addr::unchecked(reward_contract_address.clone()),
                            token_code_hash: reward_contract_code_hash.to_string(),
                        },
                    }),
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

        pub fn add_amm_pairs_no_staking(
            factory_addr: String,
            backend: &str,
            account_name: &str,
            token_0_address: String,
            token_1_address: String,
            token_code_hash: String,
            reports: &mut Vec<Report>
        ) -> io::Result<()> {
            println!("Creating New Pairs for factory {} - token_0 {} - token_1 {} - no staking", factory_addr.clone(), token_0_address.clone(),token_1_address.clone());
            let factory_contract = NetContract { label: "".to_string(), id: "".to_string(), address: factory_addr.clone(), code_hash: "".to_string() };

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
        
        pub fn list_pair_from_factory(
            factory_addr: String,
            start: u64,
            limit: u8
        ) -> io::Result<()>
        {
            let factory_contract = NetContract { label: "".to_string(), id: "".to_string(), address: factory_addr.clone(), code_hash: "".to_string() };       
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

        pub fn get_token_type(pairs: TokenPair) -> io::Result<(String,String)>{
            let token_0_address = match pairs.0 {
                TokenType::CustomToken { contract_addr, token_code_hash: _ } =>{
                    contract_addr.clone().to_string()
                },
                TokenType::NativeToken { denom: _ } => {
                    "".to_string()
                }
            };
        
            let token_1_address = match pairs.1 {
                TokenType::CustomToken { contract_addr, token_code_hash: _ } =>{
                    contract_addr.clone().to_string()
                },
                TokenType::NativeToken { denom: _ } => {
                    "".to_string()
                }
            };
        
            Ok((token_0_address, token_1_address))
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
            reports: &mut Vec<Report>
        ) -> io::Result<()>
        {
            let pair_contract=  NetContract { 
                label: "".to_string(), 
                id: "".to_string(), 
                address: pair_addr.clone(), 
                code_hash: "".to_string() };

            let pair = TokenPair(
                TokenType::CustomToken { contract_addr: Addr::unchecked(token_0_addr.clone()), token_code_hash: token_code_hash.clone() },
                TokenType::CustomToken { contract_addr: Addr::unchecked(token_1_addr.clone()), token_code_hash: token_code_hash.clone() }
            );

            let mut staking:Option<bool> = None;
            if staking_opt == true{
                staking = Some(true);
            }

            handle(
                &AMMPairHandlMsg::AddLiquidityToAMMContract {
                    deposit: TokenPairAmount {
                        pair: pair.clone(),
                        amount_0: amount_0,
                        amount_1: amount_1,
                    },
                    slippage: None,
                    staking: staking
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