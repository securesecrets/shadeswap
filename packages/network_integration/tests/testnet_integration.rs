use colored::Colorize;
use network_integration::utils::{
    generate_label, print_contract, print_header, print_vec, print_warning, ACCOUNT_KEY,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, SNIP20_FILE, STORE_GAS, VIEW_KEY,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, Report},
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
        Callback, ContractInstantiationInfo, ContractLink, secret_toolkit::snip20::{Balance, BalanceResponse},
    },
    msg::{
        amm_pair::{HandleMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, InvokeMsg},
        factory::{
            HandleMsg as FactoryHandleMsg, InitMsg as FactoryInitMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
    },
    Pagination, TokenAmount, TokenPair, TokenPairAmount, TokenType,
};
use std::env;

use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};