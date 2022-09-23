use std::{io, string};
use std::io::{Write};
use std::io::BufRead;
use secretcli::cli_types::StoredContract;
use shadeswap_shared::core::{ TokenPair, TokenPairAmount, TokenType, Fee, ViewingKey};
use shadeswap_shared::router;
use cosmwasm_std::{Uint128, Addr};
use cosmwasm_std::to_binary;
use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY, 
    STAKING_FILE, VIEW_KEY, SNIP20_FILE, InitConfig, init_contract_factory, init_snip20_cli,
};
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
};
use shadeswap_shared::{
    amm_pair::{AMMPair, AMMSettings},
    core::{ContractInstantiationInfo, ContractLink},
    msg::{
        amm_pair::{
            ExecuteMsg as AMMPairHandlMsg, InitMsg as AMMPairInitMsg, InvokeMsg,
            QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryMsgResponse,
        },
        factory::{
            ExecuteMsg as FactoryExecuteMsg, InitMsg as FactoryInitMsg,
            QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse,
        },
        router::{
            ExecuteMsg as RouterExecuteMsg, InitMsg as RouterInitMsg, InvokeMsg as RouterInvokeMsg,
            QueryMsg as RouterQueryMsg, QueryMsgResponse as RouterQueryResponse,
        },
        staking::{
            ExecuteMsg as StakingMsgHandle, QueryMsg as StakingQueryMsg,
            QueryResponse as StakingQueryMsgResponse,
        },
    },
    stake_contract::StakingContractInit,
    Pagination,
};

// pub const SNIP20_FILE: &str = "../../../compiled/snip20.wasm.gz";
// pub const LPTOKEN20_FILE: &str = "../../compiled/lp_token.wasm.gz";
// pub const AMM_PAIR_FILE: &str = "../../compiled/amm_pair.wasm.gz";
// pub const FACTORY_FILE: &str = "../../compiled/factory.wasm.gz";
// pub const ROUTER_FILE: &str = "../../compiled/router.wasm.gz";
// pub const STAKING_FILE: &str = "../../compiled/staking.wasm.gz";


pub const STORE_GAS: &str = "10000000";

fn main() -> io::Result<()> {
     
    let account_name = read_string("Account Name ")?;
    let keyring_backend = read_string("Keyring Backend ")?;
    while true 
    {      
        print_options()?;
        let input = read_input()?;
        if input == 10
        {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(b"Exiting Secretd Cli.\n\t")?; 
            handle.flush()?;             
            break;
        }

        if input == 1{
            let mut reports = vec![];   
            let contract = create_new_snip_20(&account_name.trim(), &keyring_backend.trim(), &mut reports)?;
        }

        if input == 2{
            create_new_deployment(account_name.trim(), keyring_backend.trim())?;
        }

        if input == 3{
            let mut reports = vec![];  
            add_new_pair_into_existing_factory(&account_name.trim(), &keyring_backend.trim(), &mut reports)?;
        }

        if input == 4 {
            let mut reports = vec![];  
            let contract = create_snip20_and_register(&account_name.trim(), &keyring_backend.trim(),
                &mut reports)?;
        }

        
        if input == 5 {
            let mut reports = vec![];  

            let btc_contrat = NetContract{
                label : "".to_string(),
                id : "12143".to_string(),
                address: "secret1yn7jaxaukswkrqykvnz8rs8dvc4nqqty4dut9l".to_string(),
                code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string()
            };            

            let etc_contrat = NetContract{
                label : "".to_string(),
                id : "12144".to_string(),
                address: "secret1hvgu4gt8w20j5m3vc8tjvgewff3rx88ewewyq8".to_string(),
                code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string()
            };

            let usdt_contrat = NetContract{
                label : "".to_string(),
                id : "".to_string(),
                address: "secret199mjvc9ggw9yh23lr5xya2dxw7qemeqktzw2wc".to_string(),
                code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string()
            };

            let recipient = read_string("Recipient address: ")?;
            let amount = read_amount("Amount:: ")?;
            // _ = mint_snip20(&account_name.trim(), &
            //         keyring_backend.trim(),
            //         Addr::unchecked(recipient.clone().trim()),
            //         Uint128(1000000),
            //         "1000000uscrt",
            //         &mut reports,
            //         btc_contrat.clone())?;       
            
            _ = mint_snip20(
                &account_name.trim(), 
                &keyring_backend.trim(),
                Addr::unchecked(recipient.clone().trim()),
                Uint128::from(1000000u128),
                "1000000uscrt",
                &mut reports,
                usdt_contrat.clone())?;    
            
          
            _ = mint_snip20(&account_name.trim(), &
                    keyring_backend.trim(),
                    Addr::unchecked(recipient.clone().trim()),
                    Uint128::from(1000000u128),
                    "1000000uscrt",
                    &mut reports,
                    etc_contrat.clone())?;                 
        }
    }    
    Ok(())
}

fn print_options() -> io::Result<()>
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(b"Please select list of the action would you like to perform")?;
    handle.write_all(b"\n\t1. Create New Snip20 token. ")?;
    handle.write_all(b"\n\t2. Create New Deployment. ")?;
    handle.write_all(b"\n\t3. Create New AMMPair Contract. ")?;
    handle.write_all(b"\n\t4. Create Snip20 And Create New AMM Pair. ")?;
    handle.write_all(b"\n\t5. Mint20 Snip20. \n\t")?;
    handle.write_all(b"\n\t10. Exit Secretd Cli. \n\t")?;
    handle.flush()?;
  
    Ok(())
}

fn create_new_deployment(account_name: &str, backend: &str) -> io::Result<()> 
{
    let mut reports = vec![];   
    let contract_0 = create_new_snip_20(account_name, backend, &mut reports)?;
    let contract_1 = create_new_snip_20(account_name, backend, &mut reports)?;
    let reward_token = create_new_snip_20(account_name, backend, &mut reports)?;
    let factory = create_factory_contract(account_name, backend, &mut reports)?;
   
    let router_contract = init_router_contract(factory.clone(), 
    account_name, backend, &mut reports)?;

    let staking_contract = 
    store_and_return_contract(&STAKING_FILE.replace("../", ""), 
        &account_name,
        Some(STORE_GAS),
        Some(backend)
    )?;
    let pairs = TokenPair(
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(contract_0.address.clone()),
            token_code_hash: contract_0.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(contract_1.address.clone()),
            token_code_hash: contract_1.code_hash.to_string(),
        },
    );

    register_snip20_router(account_name,backend, contract_0.clone(), router_contract.clone(), &mut reports)?;
    register_snip20_router(account_name,backend, contract_1.clone(), router_contract.clone(), &mut reports)?;

    add_amm_pairs(pairs.clone(), staking_contract, factory.clone(),backend,account_name,reward_token.clone(),&mut reports)?;
    let pair_address = get_all_pairs_from_factory(factory.clone(), contract_0.clone(),
     contract_1.clone())?;

    
     // register
 
    increase_allowance(pair_address.clone(),Uint128::from(100000000u128), contract_0.clone(),
        account_name, backend, &mut reports)?;
    increase_allowance(pair_address.clone(),Uint128::from(100000000u128), contract_1.clone(),
        account_name, backend, &mut reports)?;

    mint_snip20(account_name, backend, Addr::unchecked(""),
                Uint128::from(100000u128), "100000uscrt",&mut reports, contract_0.clone())?;

    mint_snip20(account_name, backend, Addr::unchecked(""),
                Uint128::from(100000u128), "100000uscrt",&mut reports, contract_1.clone())?;
    
    add_liquidity(pairs.clone(), 
    NetContract { 
        label: "".to_string(),
        id: "".to_string(), 
        address: pair_address.clone(), 
        code_hash: "".to_string()
    },   
    account_name,
    backend,
    Uint128::from(100000u128),
    Uint128::from(100000u128),
    &mut reports
    )?;

    println!("Results...\n\t");
    println!("Token 0 Address {}", contract_0.address.clone().to_string());
    println!("Token 0 Id {}", contract_0.id.clone().to_string());
    println!("Token 0 Code Hash {}", contract_0.code_hash.clone().to_string());
    println!("Token 1 Address {}", contract_1.address.clone().to_string());
    println!("Token 1 Id {}", contract_1.id.clone().to_string());
    println!("Token 1 Code Hash {}", contract_1.code_hash.clone().to_string());
    println!("Factory Address {}", factory.address.clone().to_string());
    println!("Factory Id {}", factory.id.clone().to_string());
    println!("Factory Code Hash {}", factory.code_hash.clone().to_string());
    println!("Router Address {}", router_contract.address.clone().to_string());
    println!("Router Id {}", router_contract.id.clone().to_string());
    println!("Router Code Hash {}", router_contract.code_hash.clone().to_string());
    println!("Pair Address {:?}", pair_address);
   
    Ok(())
}


fn read_string(text: &str)-> io::Result<String>
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut question ="Please type ".to_string();
    question.push_str(text);
    handle.write_all(question.as_bytes())?;
    handle.flush()?;

    // let stdout = io::stdout();
    // let mut handle = stdout.lock();
    // handle.flush()?;
    let mut output = String::new();
    std::io::stdin().read_line(&mut output).unwrap();    
    Ok(output)
}

fn read_int(text: &str) -> io::Result<u8>
{   
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut question ="Please type ".to_string();
    question.push_str(text);
    handle.write_all(question.as_bytes())?;
    handle.flush()?;

    println!("Please choose option.");
    let option: u8 = std::io::stdin()
    .lock()
    .lines()
    .next()
    .expect("stdin should be available")
    .expect("couldn't read from stdin")
    .trim()
    .parse()
    .expect("input was not an integer"); 
    Ok(option)
}

fn read_amount(text: &str) -> io::Result<u128>
{   
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut question ="Please type ".to_string();
    question.push_str(text);
    handle.write_all(question.as_bytes())?;
    handle.flush()?;

    println!("Please choose option.");
    let option: u128 = std::io::stdin()
    .lock()
    .lines()
    .next()
    .expect("stdin should be available")
    .expect("couldn't read from stdin")
    .trim()
    .parse()
    .expect("input was not an integer"); 
    Ok(option)
}

fn read_input() -> io::Result<i32>
{   
    println!("Please choose option.");
    let option: i32 = std::io::stdin()
    .lock()
    .lines()
    .next()
    .expect("stdin should be available")
    .expect("couldn't read from stdin")
    .trim()
    .parse()
    .expect("input was not an integer"); 
    Ok(option)
}

fn create_new_snip_20(account_name: &str, backend: &str, reports: &mut Vec<Report>) -> io::Result<NetContract>
{   
    let name = read_string(&"Type Name Snip20:: ".to_string())?;
    let symbol = read_string(&"Type Symbol Snip20:: ".to_string())?;
    let decimals = read_int(&"Type decimals:: ".to_string())?;
    let snip20 = init_snip20_contract(&name.trim(), &symbol.trim(),
    reports, decimals, account_name, backend)?;

     let contract = NetContract{
        label: snip20.label.to_string(),
        id: snip20.id.clone().to_string(),
        code_hash: snip20.code_hash.clone(),
        address: snip20.address.clone().to_string()
    };
    
    set_viewing_key(VIEW_KEY.to_owned(), &contract.clone(), reports,
        account_name.trim(), backend.trim())?;
    Ok(contract)
}

fn init_snip20_contract(symbol: &str, name: &str, reports: &mut Vec<Report>, 
    decimal: u8, account_name: &str, keyring_backend: &str) -> io::Result<NetContract>{
      
    let config = InitConfig{
        enable_burn: Some(true),
        enable_mint: Some(true),
        enable_deposit : Some(true),
        enable_redeem: Some(true),
        public_total_supply: Some(true),
    };

    let s_contract = init_snip20_cli(
        name.to_string(),
        symbol.to_string(),
        8, //decimal,
        Some(config),
        reports,
        &account_name,
        Some(&SNIP20_FILE.replace("../", "")),
        &keyring_backend        
    )?;

    println!("Contract address - {}", s_contract.1.address.clone());
    println!("Code hash - {}", s_contract.1.code_hash.clone());
    println!("Code Id - {}", s_contract.1.id);
    
    Ok(s_contract.1)
}

fn add_new_pair_into_existing_factory(account_name: &str,
    backend:&str,
    reports: &mut Vec<Report>) -> io::Result<()>
{
    let staking_contract = 
    store_and_return_contract(&STAKING_FILE.replace("../", ""), 
        &account_name,
        Some(STORE_GAS),
        Some(backend)
    )?;
    // let factory_contract = NetContract{
    //     address : "secret15llpx5ahfhadk29pm9qswe0ssw6x7pkpkhvnj5".to_string(),
    //     code_hash: "71EB188450FBE579E5601AF81D8416890E502BA3A2799B693185B304239F2E20" .to_string(),
    //     label: "".to_string(),
    //     id: "12148".to_string()
    // };

    let factory_contract = NetContract{
        address : "secret1tzgh5rd8v6rk2telfleaf3k55nzgvzm92zz93s".to_string(),
        code_hash: "71EB188450FBE579E5601AF81D8416890E502BA3A2799B693185B304239F2E20" .to_string(),
        label: "".to_string(),
        id: "12156".to_string()
    };

    let reward_token = create_new_snip_20(account_name, backend, reports)?;
    let contract_0 = create_new_snip_20(account_name, backend, reports)?;

    // let contract_1 =  NetContract{
    //     id: "12144".to_string(),
    //     address: "secret1hvgu4gt8w20j5m3vc8tjvgewff3rx88ewewyq8".to_string(),
    //     code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string(),
    //     label: "".to_string(),
    // };

    let contract_1 =  NetContract{
        id: "12144".to_string(),
        address: "secret1092ufc3ur5538y7cv4cr3ksu6v8k7t30l2ahvy".to_string(),
        code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string(),
        label: "".to_string(),
    };    

    let router_contract = NetContract {
        id: "12157".to_string(),
        address: "secret12yyrnm0y3duxzelge2nexythvvc03jthx6qr6h".to_string(),
        code_hash: "5ABF0B785628E960511C3A2AD0F27C0BDD48E8D4C8EAE3150562701D72772AF4".to_string(),
        label: "".to_string()
    };

    register_snip20_router(account_name, backend, contract_0.clone(), router_contract.clone(), reports)?;

    let pairs = TokenPair(
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(contract_0.address.clone()),
            token_code_hash: contract_0.code_hash.to_string(),
        },
        TokenType::CustomToken {
            contract_addr: Addr::unchecked(contract_1.address.clone()),
            token_code_hash: contract_1.code_hash.to_string(),
        },
    );

    create_amm_pair_and_set_liquidity(pairs.clone(), account_name, backend, reward_token.clone(),
        reports, staking_contract, contract_1.clone(), contract_1.clone(), factory_contract.clone())?;
        
    Ok(())
}



fn create_amm_pair_and_set_liquidity(
    pairs: TokenPair,
    account_name: &str,
    backend: &str,
    reward_token: NetContract,
    reports: &mut Vec<Report>,
    staking_contract: StoredContract,
    contract_token_0: NetContract,
    contract_token_1: NetContract,
    factory_contract: NetContract,
) -> io::Result<()>{    

    add_amm_pairs(pairs.clone(), staking_contract, factory_contract.clone(),backend,account_name,reward_token.clone(), reports)?;
    let pair_address = get_all_pairs_from_factory(factory_contract.clone(), contract_token_0.clone(),
    contract_token_1.clone())?;      
   
    increase_allowance(pair_address.clone(),Uint128::from(100000000u128), contract_token_0.clone(),
        account_name, backend, reports)?;
    increase_allowance(pair_address.clone(),Uint128::from(100000000u128), contract_token_1.clone(),
        account_name, backend, reports)?;

    mint_snip20(account_name, backend, Addr::unchecked("secret10x73565sfrn8veawzz69le0na4y5nluqlwdvqu"),
                Uint128::from(100000000u128), "100000000uscrt", reports, contract_token_0.clone())?;

    mint_snip20(account_name, backend, Addr::unchecked("secret10x73565sfrn8veawzz69le0na4y5nluqlwdvqu"),
                Uint128::from(100000000u128), "100000000uscrt", reports, contract_token_1.clone())?;
    
    add_liquidity(pairs.clone(), 
    NetContract { 
        label: "".to_string(),
        id: "".to_string(), 
        address: pair_address.clone(), 
        code_hash: "".to_string()
    },   
    account_name,
    backend,
    Uint128::from(100000000u128),
    Uint128::from(100000000u128),
    reports
    )?;

    println!("Results...\n\t");
    println!("Token 0 Address {}", contract_token_0.address.clone().to_string());
    println!("Token 0 Id {}", contract_token_0.id.clone().to_string());
    println!("Token 0 Code Hash {}", contract_token_0.code_hash.clone().to_string());
    println!("Token 1 Address {}", contract_token_1.address.clone().to_string());
    println!("Token 1 Id {}", contract_token_1.id.clone().to_string());
    println!("Token 1 Code Hash {}", contract_token_1.code_hash.clone().to_string());
    println!("Factory Address {}", factory_contract.address.clone().to_string());
    println!("Factory Id {}", factory_contract.id.clone().to_string());
    println!("Factory Code Hash {}", factory_contract.code_hash.clone().to_string());   
    println!("Pair Address {:?}", pair_address);
    Ok(())
}

fn create_factory_contract(account_name: &str, backend: &str, reports: &mut Vec<Report>) 
-> io::Result<NetContract>
{
    let lp_token = 
        store_and_return_contract(&LPTOKEN20_FILE.replace("../", ""), 
            &account_name,
            Some(STORE_GAS),
            Some(backend)
        )?;

     let pair_contract = 
        store_and_return_contract(&AMM_PAIR_FILE.replace("../", ""), 
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
    };

    println!("Creating Factory ");
    let factory_contract = init_contract_factory(
        &account_name, 
        &backend,
        &FACTORY_FILE.replace("../", ""), 
        &init_msg, 
        reports
    )?;
   
    println!("Factory address {}", factory_contract.address.clone().to_string());
    Ok(factory_contract)
}

fn add_amm_pairs(pairs: TokenPair,
    staking_contract: StoredContract,
    factory_contract: NetContract,
    backend: &str,
    account_name: &str,
    reward_contract: NetContract,
    reports: &mut Vec<Report>
) -> io::Result<()> {
    handle(
        &FactoryExecuteMsg::CreateAMMPair {
            pair: pairs.clone(),
            entropy: to_binary(&"".to_string()).unwrap(),           
            staking_contract: Some(StakingContractInit {
                contract_info: ContractInstantiationInfo {
                    code_hash: staking_contract.code_hash.to_string(),
                    id: staking_contract.id.clone().parse::<u64>().unwrap(),
                },
                amount: Uint128::from(3450000000000u128),
                reward_token: TokenType::CustomToken {
                    contract_addr: Addr::unchecked(reward_contract.address.clone()),
                    token_code_hash: reward_contract.code_hash.to_string(),
                },
            }),
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

fn set_viewing_key(
    viewingKey: String, 
    netContract: &NetContract, 
    reports: &mut Vec<Report>,
    account_name: &str,
    backend: &str) ->io::Result<()>{
    let msg = snip20_reference_impl::msg::ExecuteMsg::SetViewingKey {
        key: String::from(VIEW_KEY),
        padding: None,
    };

    handle(
        &msg,
        &netContract,
        account_name,
        Some(GAS),
        Some(backend),
        None,
        reports,
        None,
    )?;
    Ok(())
}


fn get_all_pairs_from_factory(
    factory_contract: NetContract,
    token_0_contract: NetContract,
    token_1_contract: NetContract
) -> io::Result<String>
{
    let mut pair_address = "".to_string();
    let mut start = 0;
    let mut limit = 30;

    let msg = FactoryQueryMsg::ListAMMPairs {
        pagination: Pagination {
            start: start,
            limit: limit,
        },
    };
    let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
    if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
        for pair in amm_pairs.iter()
        {
            for i in 0..amm_pairs.len() {
                println!("{:?}", amm_pairs[i]);

                let pair = amm_pairs[i].clone();
                let token_type = get_token_type(pair.pair)?;
                if token_type.0 == token_0_contract.address && token_1_contract.address == token_type.1
                {                  
                    pair_address = pair.address.clone().to_string();
                }        
            }
        }          
    }

    Ok(pair_address)
}

pub fn get_token_type(pairs: TokenPair) -> io::Result<(String,String)>{
    let token_0_address = match pairs.0 {
        TokenType::CustomToken { contract_addr, token_code_hash } =>{
            contract_addr.clone().to_string()
        },
        TokenType::NativeToken { denom } => {
            "".to_string()
        }
    };

    let token_1_address = match pairs.1 {
        TokenType::CustomToken { contract_addr, token_code_hash } =>{
            contract_addr.clone().to_string()
        },
        TokenType::NativeToken { denom } => {
            "".to_string()
        }
    };

    Ok((token_0_address, token_1_address))
}

pub fn init_router_contract(pair_contract: NetContract,
account_name: &str,
backend: &str,
reports: &mut Vec<Report>) -> io::Result<NetContract>
{
    let router_msg = RouterInitMsg {
        prng_seed: to_binary(&"".to_string()).unwrap(),      
        entropy: to_binary(&"".to_string()).unwrap(),
        viewing_key: Some(ViewingKey::from(VIEW_KEY).to_string()),
        pair_contract_code_hash: pair_contract.code_hash,
    };

    let router_contract = init(
        &router_msg,
        &ROUTER_FILE.replace("../", ""),
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
    contract_snip20: NetContract,
    router_contract: NetContract,
    reports: &mut Vec<Report>
) -> io::Result<()>
{
    handle(
        &RouterExecuteMsg::RegisterSNIP20Token {
            token_addr: Addr::unchecked(contract_snip20.address.clone()),
            token_code_hash: contract_snip20.code_hash.to_string(),        
        },
        &router_contract,
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

pub fn create_snip20_and_register(
    account_name: &str,
    backend: &str,
    reports: &mut Vec<Report>
) -> io::Result<NetContract>
{
    let contract_0 = create_new_snip_20(account_name, backend, reports)?; 
    let router_contract = NetContract {
        id: "12157".to_string(),
        address: "secret12yyrnm0y3duxzelge2nexythvvc03jthx6qr6h".to_string(),
        code_hash: "5ABF0B785628E960511C3A2AD0F27C0BDD48E8D4C8EAE3150562701D72772AF4".to_string(),
        label: "".to_string()
    };

    register_snip20_router(account_name, backend, contract_0.clone(), router_contract.clone(), reports)?;
    Ok(contract_0)
}

pub fn increase_allowance(
    pair_address: String,
    amount: Uint128,
    contract_snip20: NetContract,
    account_name: &str,
    backend: &str,
    reports: &mut Vec<Report>
) -> io::Result<()>
{
    handle(
        &snip20_reference_impl::msg::ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked(String::from(pair_address)),
            amount: amount, // Uint128(100000000),
            expiration: None,
            padding: None,
        },
        &NetContract {
            label: "".to_string(),
            id: contract_snip20.id.clone(),
            address: contract_snip20.address.clone(),
            code_hash: contract_snip20.code_hash.to_string(),
        },
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

pub fn mint_snip20(
    account_name: &str,
    backend: &str,
    recipient: Addr,
    amount: Uint128, 
    amount_uscrt: &str, 
    reports: &mut Vec<Report>,
    contract_snip20: NetContract
) -> io::Result<()>{
    let msg = snip20_reference_impl::msg::ExecuteMsg::Mint { padding: None, recipient: recipient, amount: amount, memo: None };
    handle(
        &msg,
        &contract_snip20,
        account_name,
        Some(GAS),
        Some(backend),
        Some(amount_uscrt),
        reports,
        None,
    )?;
    Ok(())
}

pub fn add_liquidity(
    pair: TokenPair,
    pair_contract: NetContract,
    account_name: &str,
    backend: &str,
    amount_0: Uint128, 
    amount_1: Uint128, 
    reports: &mut Vec<Report>
) -> io::Result<()>
{
    handle(
        &AMMPairHandlMsg::AddLiquidityToAMMContract {
            deposit: TokenPairAmount {
                pair: pair.clone(),
                amount_0: amount_0,
                amount_1: amount_1,
            },
            slippage: None,
            staking: None
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