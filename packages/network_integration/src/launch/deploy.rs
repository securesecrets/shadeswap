use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    AMM_PAIR_FILE, STAKING_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY,
    SNIP20_FILE, VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
};
use serde_json::Result;
use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings, Fee},
    fadroma::{
        scrt::{
            from_binary, log, secret_toolkit::snip20, to_binary, Api, BankMsg, Binary, Coin,
            CosmosMsg, Decimal, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
            QueryRequest, QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
        },
        secret_toolkit::snip20::{Balance, BalanceResponse},
        Callback, ContractInstantiationInfo, ContractLink, ViewingKey,
    },
    stake_contract::StakingContractInit,
    msg::{
        amm_pair::{HandleMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg,QueryMsgResponse as AMMPairQueryMsgResponse ,
             QueryMsg as AMMPairQueryMsg, InvokeMsg},
        factory::{
            HandleMsg as FactoryHandleMsg, InitMsg as FactoryInitMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
        router::{
            HandleMsg as RouterHandleMsg, InitMsg as RouterInitMsg, InvokeMsg as RouterInvokeMsg,
        },
    },
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType,
};
use std::env;

use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

pub const ACCOUNT_KEY: &str  = "deployer";
pub const STORE_GAS: &str  = "10000000";

fn main() -> serde_json::Result<()> {

    print_header("Storing all contracts");
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    let mut reports = vec![];
    print_warning("Storing LP Token Contract");
    let s_lp = 
        store_and_return_contract(&LPTOKEN20_FILE.replace("../",""), ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    print_warning("Storing AMM Pair Token Contract");
    let s_ammPair =
        store_and_return_contract(&AMM_PAIR_FILE.replace("../",""), ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    
    print_warning("Storing Staking Contract");
    let staking_contract = 
        store_and_return_contract(&STAKING_FILE.replace("../",""), ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;

    print_header("\n\tInitializing Factory Contract");

    let factory_msg = FactoryInitMsg {
        pair_contract: ContractInstantiationInfo {
            code_hash: s_ammPair.code_hash.to_string(),
            id: s_ammPair.id.clone().parse::<u64>().unwrap(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(8, 100),
            shade_dao_fee: Fee::new(2, 100),
            shade_dao_address: ContractLink {
                address: HumanAddr(String::from("".to_string())),
                code_hash: "".to_string(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: s_lp.code_hash.clone(),
            id: s_lp.id.clone().parse::<u64>().unwrap(),
        },
        prng_seed: to_binary(&"".to_string()).unwrap(),
    };

    let factory_contract = init(
        &factory_msg,
        &FACTORY_FILE.replace("../",""),
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;

    print_contract(&factory_contract);


    print_header("\n\tGetting Pairs from Factory");
    {
        let msg = FactoryQueryMsg::ListAMMPairs {
            pagination: Pagination {
                start: 0,
                limit: 10,
            },
        };

        let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
        if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
            assert_eq!(amm_pairs.len(), 0);


            print_header("\n\tInitializing Router");

            let router_msg = RouterInitMsg {
                prng_seed: to_binary(&"".to_string()).unwrap(),
                factory_address: ContractLink {
                    address: HumanAddr(String::from(factory_contract.address)),
                    code_hash: factory_contract.code_hash,
                },
                entropy: to_binary(&"".to_string()).unwrap(),
                viewing_key: Some(ViewingKey::from(VIEW_KEY)),
            };

            let router_contract = init(
                &router_msg,
                &ROUTER_FILE.replace("../",""),
                &*generate_label(8),
                ACCOUNT_KEY,
                Some(STORE_GAS),
                Some(GAS),
                Some("test"),
                &mut reports,
            )?;
            print_contract(&router_contract);
                

        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    return Ok(());
}