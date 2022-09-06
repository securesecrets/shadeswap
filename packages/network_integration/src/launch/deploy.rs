use network_integration::utils::InitConfig;
use shadeswap_shared::viewing_keys::ViewingKey;
use shadeswap_shared::custom_fee::Fee;
use cosmwasm_std::Uint128;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::to_binary;
use colored::Colorize;
use network_integration::utils::{
    generate_label, init_snip20, print_contract, print_header, print_vec, print_warning,
    AMM_PAIR_FILE, FACTORY_FILE, GAS, LPTOKEN20_FILE, ROUTER_FILE, SHADE_DAO_KEY, SNIP20_FILE,
    STAKING_FILE, VIEW_KEY,
};

use cosmwasm_std::BalanceResponse;
use secretcli::{
    cli_types::NetContract,
    secretcli::{account_address, handle, init, query, store_and_return_contract, Report},
};

use serde_json::Result;
use shadeswap_shared::core::ContractInstantiationInfo;
use shadeswap_shared::secret_toolkit::snip20::HandleMsg;
use shadeswap_shared::secret_toolkit::snip20::QueryMsg;
use shadeswap_shared::{
    amm_pair::{AMMPair},
    msg::{
        amm_pair::{
            HandleMsg as AMMPairHandlMsg,
        },
        factory::{
            HandleMsg as FactoryHandleMsg, QueryMsg as FactoryQueryMsg,
            QueryResponse as FactoryQueryResponse,
        },
        router::{
            HandleMsg as RouterHandleMsg,
        },
    },
    stake_contract::StakingContractInit,
    Pagination, TokenPair, TokenPairAmount, TokenType,
};



pub const ACCOUNT_KEY: &str = "deployer";
pub const STORE_GAS: &str = "10000000";

pub fn get_balance(contract: &NetContract, from: String, view_key: String) -> Uint128 {
    let msg = QueryMsg::Balance {
        address: HumanAddr::from(from),
        key: view_key,
    };

    let balance: BalanceResponse = query(contract, &msg, None).unwrap();

    balance.amount.amount
}

fn main() -> serde_json::Result<()> {
    let mut reports = vec![];

    // let contract_eth = init_snip20_contract("ETH".to_string(), "ETH".to_string(), &mut reports, 18)?;
    // let contract_btc = init_snip20_contract("BTC".to_string(), "BTC".to_string(), &mut reports, 8)?;    
    let reward_contract = init_snip20_contract("REWARD".to_string(), "Reward BTC/ETH".to_string(), &mut reports, 18)?; 
    // set_viewing_key(VIEW_KEY.to_owned(), &contract_btc,&mut reports);
    // set_viewing_key(VIEW_KEY.to_owned(), &contract_eth, &mut reports);
    set_viewing_key(VIEW_KEY.to_owned(), &reward_contract, &mut reports);

    let router_contract = NetContract{
        label: "vx7MHIip".to_string(),
        id: "11426".to_string(),
        address: "secret18letgdtj6fz55u4a9fm5hal9tez3ruz79gscpj".to_string(),
        code_hash: "5ABF0B785628E960511C3A2AD0F27C0BDD48E8D4C8EAE3150562701D72772AF4".to_string()
    };

    let factory_contract = NetContract{
        label: "I7nZ28Aq".to_string(),
        id: "11425".to_string(),
        address: "secret1fxnrc2qda4c7p2qsu6k0yuu9rsaqdez54zr0q3".to_string(),
        code_hash: "71EB188450FBE579E5601AF81D8416890E502BA3A2799B693185B304239F2E20".to_string()
    };

    let contract_eth_created = NetContract{
        label: "".to_string(),
        id: "12011".to_string(),
        address: "secret1hp077v3asc8uncktxlccpm5rrur8zkes0ln3ng".to_string(),
        code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string()
    };

    let contract_btc_created = NetContract{
        label: "".to_string(),
        id: "12012".to_string(),
        address: "secret17mm7szc7wem5fhlp8xf2x2fhwl84d98l7tamd8".to_string(),
        code_hash: "DFE53F3E3FFF02E59077AD4986FF6B620B084E2B3A4E9CE133155EABE105FFF1".to_string()
    };

    print_header("Storing all contracts");
    let entropy = to_binary(&"ENTROPY".to_string()).unwrap();

    print_warning("Storing LP Token Contract");
    let s_lp = store_and_return_contract(
        &LPTOKEN20_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;
    print_warning("Storing AMM Pair Token Contract");
    let s_ammPair = store_and_return_contract(
        &AMM_PAIR_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;
    print_warning("Storing Staking Contract");
    let staking_contract = store_and_return_contract(
        &STAKING_FILE.replace("../", ""),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some("test"),
    )?;

    let btc_eth_pair = TokenPair::<HumanAddr>(
        TokenType::CustomToken {
            contract_addr: contract_btc_created.address.to_string().into(),
            token_code_hash: contract_btc_created.code_hash.clone(),
        },
        TokenType::CustomToken {
            contract_addr: contract_eth_created.address.to_string().into(),
            token_code_hash: contract_eth_created.code_hash.clone(),
        },
    );

    print_contract(&factory_contract);
    print_contract(&router_contract);

    
    print_header("Creating AMM PAIRS");
    handle(
                &FactoryHandleMsg::CreateAMMPair {
                    pair: btc_eth_pair.clone(),
                    entropy: entropy,
                    staking_contract: Some(StakingContractInit {
                        contract_info: ContractInstantiationInfo {
                            code_hash: staking_contract.code_hash.to_string(),
                            id: staking_contract.id.clone().parse::<u64>().unwrap(),
                        },
                        amount: Uint128(100000u128),
                        reward_token: TokenType::CustomToken {
                            contract_addr: reward_contract.address.clone().into(),
                            token_code_hash: reward_contract.code_hash.to_string(),
                        },
                    }),
                },
                &factory_contract,
                ACCOUNT_KEY,
                Some(GAS),
                Some("test"),
                None,
                &mut reports,
                None,
            )
            .unwrap();             
           

           print_header("\n\tGetting Pairs from Factory");
        
                let msg = FactoryQueryMsg::ListAMMPairs {
                    pagination: Pagination {
                        start: 0,
                        limit: 30,
                    },
                };

                let factory_query: FactoryQueryResponse = query(&factory_contract, msg, None)?;
                if let FactoryQueryResponse::ListAMMPairs { amm_pairs } = factory_query {
                    for i in 0..amm_pairs.len(){
                        print!("AMM Pairs address : {}", amm_pairs[i].address);
                        print!("AMM Pairs pairs : {}", amm_pairs[i].pair.0);
                    }

                    let index = amm_pairs.len();
                    let ammPair = amm_pairs[index].clone();    
                                        
                    increase_allowance(ammPair.clone(), contract_btc_created.clone(), Uint128(100000000), &mut reports)?;
                    increase_allowance(ammPair.clone(), contract_eth_created.clone(), Uint128(100000000), &mut reports)?;

                    add_mint(HumanAddr::from("secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q"), &contract_btc_created.clone(), "10000000uscrt".clone(), &mut reports,Uint128(10000000))?;
                    add_mint(HumanAddr::from("secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q"), &&contract_eth_created.clone(), "10000000uscrt".clone(), &mut reports, Uint128(10000000))?;
                    add_liquidity(
                        &NetContract { 
                            label: "".to_string(), 
                            id: s_ammPair.id.to_string(), 
                            address: ammPair.address.to_string(), 
                            code_hash: s_ammPair.code_hash.to_string()
                        },
                        btc_eth_pair.clone(),
                        &mut reports,
                        Uint128(10000),
                        Uint128(10000)
                    )?;               
            }             
    Ok(())
}

fn set_viewing_key(viewingKey: String, netContract: &NetContract, reports: &mut Vec<Report>) ->Result<()>{
    let msg = snip20_reference_impl::msg::HandleMsg::SetViewingKey {
        key: String::from(VIEW_KEY),
        padding: None,
    };
    handle(
        &msg,
        &netContract,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?;

    Ok(())
}

fn init_snip20_contract(symbol: String, name: String, reports: &mut Vec<Report>, 
    decimal: u8) -> Result<NetContract>{
    
    let test = String::from(name.clone() + "Initializing");
    print_header(&test);   
         
    let config = InitConfig{
        enable_burn: Some(true),
        enable_mint: Some(true),
        enable_deposit : Some(true),
        enable_redeem: Some(true),
        public_total_supply: Some(true),
    };

    let (s_sSHDINIT, s_contract) = init_snip20(
        name.to_string(),
        symbol.to_string(),
        decimal,
        Some(config),
        reports,
        ACCOUNT_KEY,
        Some(&SNIP20_FILE.replace("../", "")),
    )?;

    println!("Contract address - {}", s_contract.address.clone());
    println!("Code hash - {}", s_contract.code_hash.clone());
    println!("Code Id - {}", s_contract.id);
    
    Ok(s_contract)
}

  
fn register_snip20_to_router(contract_snip20: &NetContract, 
    router_contract: &NetContract, 
    reports: &mut Vec<Report>
) -> Result<()>
{
    print_header("\n\tRegister  BTC to router");
    handle(
        &RouterHandleMsg::RegisterSNIP20Token {
            token: HumanAddr::from(contract_snip20.address.clone()),
            token_code_hash: contract_snip20.code_hash.to_string(),
        },
        &router_contract,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )
    .unwrap();

    Ok(())
}

fn increase_allowance(ammPair: AMMPair<HumanAddr>, contract_snip20: NetContract,
    amount: Uint128, reports: &mut Vec<Report>) ->  Result<()>{
    println!("\n\tIncreasing Allowance : {}", amount);
    handle(
        &snip20_reference_impl::msg::HandleMsg::IncreaseAllowance {
            spender: HumanAddr(String::from(ammPair.address.0.to_string())),
            amount: amount,
            expiration: None,
            padding: None,
        },
        &contract_snip20,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )
    .unwrap();
    
   Ok(())
}

fn add_liquidity(amm_pair: &NetContract, token_pair: TokenPair<HumanAddr>, reports: &mut Vec<Report>, 
    amount_0: Uint128, amount_1: Uint128) 
    -> Result<()>
{
    println!("Adding Liquidity to Pair Contract : {}", amm_pair.address.clone());
    handle(
        &AMMPairHandlMsg::AddLiquidityToAMMContract {
            deposit: TokenPairAmount {
                pair: token_pair.clone(),
                amount_0: amount_0,
                amount_1: amount_1,
            },
            slippage: None,

            staking: None
        },
        amm_pair,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )
    .unwrap();

    Ok(())
}

fn add_mint(address:HumanAddr, contract_mint: &NetContract, amount: &str, reports: &mut Vec<Report>,
mint_amount: Uint128) -> Result<()>
{
    let msg = snip20_reference_impl::msg::HandleMsg::Mint { padding: None, recipient: address, amount: mint_amount, memo: None };
       
    print_header("Depositing 10000000uscrt");
    handle(
        &msg,
        contract_mint,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        Some(amount),
        reports,
        None,
    )?;

    Ok(())
}