use std::io::{self, Write, Error, ErrorKind};

use secretcli::cli_types::StoredContract;
use secretcli::{secretcli::Report, cli_types::NetContract};
use shadeswap_shared::c_std::Uint128;

use crate::cli_commands::amm_pair_lib::{store_amm_pair, store_staking_contract, add_amm_pairs_with_staking, list_pair_from_factory, add_amm_pairs_no_staking, add_liquidity, set_reward_token};
use crate::cli_commands::snip20_lib::{create_new_snip_20, balance_snip20_query, set_viewing_key};
use crate::cli_commands::factory_lib::{create_factory_contract, mint_snip20, increase_allowance, deposit_snip20};
use crate::cli_commands::router_lib::{create_router_contract, register_snip20_router};
pub const HELP: &str = "help";
pub const CMDCREATESNIP20: &str = "snip20";
pub const CMDCREATEFACTORY: &str = "factory";
pub const CMDCREATEROUTER: &str = "router";
pub const CMDSTOREAMMPAIR: &str = "store_amm_pair";
pub const CMDSTORESTAKINGCONTRACT: &str = "store_stake";
pub const CMDREGISTERSNIP20: &str = "reg_snip20";
pub const CMDMINTSNIP20: &str = "mint_snip20";
pub const CMDINCREASEALLOWENCESNIP20: &str = "allow_snip20";
pub const CMDADDAMMPAIRS: &str = "add_amm_pair";
pub const CMDLISTAMMPAIR: &str = "list_amm_pair";
pub const CMDADDLIQ: &str = "add_liq";
pub const CMDBALANCE: &str = "snip20_bal";
pub const CMDSETREWARDTOKEN: &str = "set_reward_token";
pub const CMDDEPOSITSNIP20: &str = "deposit";
pub const CMDSETVIEWINGKEY: &str = "set_viewing_key";

pub fn parse_args(args: &[String], reports: &mut Vec<Report>) -> io::Result<()>
{
    if args.len() == 0 {
        return Err(Error::new(ErrorKind::Other, "not enough arguments"));
    }  

    let args_command = args[1].clone();
    println!("{}", args_command);
    if args_command == HELP{
        print_help()?;
    }
   
    if args_command == CMDCREATESNIP20 {
        if args.len() != 13 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();
        let name = args[4].clone();
        let symbol = args[5].clone();
        let decimal = args[6].clone();
        let viewing_key = args[7].clone();
        let enable_redeem = args[8].clone().parse::<bool>().unwrap();
        let enable_deposit = args[9].clone().parse::<bool>().unwrap();
        let enable_burn = args[10].clone().parse::<bool>().unwrap();
        let enable_mint = args[11].clone().parse::<bool>().unwrap();
        let public_total_supply = args[12].clone().parse::<bool>().unwrap();
        let snip20: NetContract = create_new_snip_20(&account_name, &backend, &name, &symbol,decimal.parse::<u8>().unwrap(), &viewing_key, reports,
        enable_burn.clone(),enable_mint.clone(),enable_deposit.clone(),enable_redeem.clone(),public_total_supply.clone())?;
        print_contract_details_cli(snip20, "Snip20".to_string());        
    }

    if args_command == CMDCREATEFACTORY {
        if args.len() != 4 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();      
        let factory: NetContract = create_factory_contract(&account_name, &backend, reports)?;
        print_contract_details_cli(factory, "Factory".to_string());
    }

    if args_command == CMDCREATEROUTER{
        if args.len() != 6 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone(); 
        let _viewing_key = args[4].clone();    
        let code_hash = args[5].clone(); 
        let router: NetContract = create_router_contract(code_hash.clone(), &account_name, &backend,  reports)?;
        print_contract_details_cli(router, "Router".to_string());
    }

    if args_command == CMDSETVIEWINGKEY{
        if args.len() != 6 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone(); 
        let token_addr = args[4].clone();    
        let viewing_key = args[5].clone(); 
        let net_contract = NetContract{
            label: "".to_string(),
            id: "".to_string(),
            address: token_addr,
            code_hash: "".to_string(),
        };
        set_viewing_key(&viewing_key,&net_contract,reports, &account_name, &backend)?;
    }

    if args_command == CMDDEPOSITSNIP20{
        if args.len() != 6 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone(); 
        let token_addr = args[4].clone();    
        let amount = args[5].clone(); 
        deposit_snip20(&account_name,&backend,&token_addr,&amount,reports)?;
    }

    if args_command == CMDSTOREAMMPAIR{
        if args.len() != 4 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();     
        let router: StoredContract = store_amm_pair(&account_name, &backend)?;
        print_stored_contract_details_cli(router, "Router".to_string());
    }

    if args_command == CMDREGISTERSNIP20{
        if args.len() != 7 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();     
        let snip20_addr = args[4].clone();
        let snip20_code_hash = args[5].clone();
        let router_address = args[6].clone();
        register_snip20_router(&account_name, &backend,snip20_addr.clone(),
            snip20_code_hash.clone(), router_address.clone(),reports)?;
        println!("Pair {} has been registered to the Router {}", snip20_addr, router_address);        
    }    

    if args_command == CMDMINTSNIP20{
        if args.len() != 8 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();     
        let snip20_addr = args[4].clone();
        let recipient = args[5].clone();
        let amount = args[6].clone();
        let amount_scrt = args[7].clone();
        let amount_u128 = amount.parse::<u128>().unwrap();
        mint_snip20(&account_name, &backend,recipient.clone(),
            Uint128::from(amount_u128), &amount_scrt.clone(),reports, snip20_addr.clone())?;
        println!("Mint SNIP20 {} has been completed", snip20_addr);        
    }

    if args_command == CMDINCREASEALLOWENCESNIP20{
        if args.len() != 7 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 
        let account_name = args[2].clone();
        let backend = args[3].clone();     
        let snip20_addr = args[4].clone();
        let spender = args[5].clone();
        let amount = args[6].clone();
        let amount_u128 = amount.parse::<u128>().unwrap();
        increase_allowance(spender.clone(), Uint128::from(amount_u128), snip20_addr.clone(), &account_name, &backend,reports)?;
        println!("Increase Allowance SNIP20 {} has been completed", snip20_addr);        
    }

    if args_command == CMDSETREWARDTOKEN{
        let account_name = args[2].clone();
        let backend = args[3].clone();  
        let staking_addr = args[4].clone();
        let reward_token_addr = args[5].clone();
        let reward_token_hash  = args[6].clone();
        let daily_reward_amount = args[5].clone().parse::<u128>().unwrap();
        let valid_to = args[6].clone().parse::<u128>().unwrap();
        set_reward_token(
            &account_name, 
            &backend,
            &staking_addr,
            &reward_token_addr, 
            &reward_token_hash,
            Uint128::new(daily_reward_amount),
            Uint128::new(valid_to),
            reports)?;
    }

    if args_command == CMDSTORESTAKINGCONTRACT{
        if args.len() != 4 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();     
        let router: StoredContract = store_staking_contract(&account_name, &backend)?;
        print_stored_contract_details_cli(router, "Staking".to_string());
    }

    if args_command == CMDADDAMMPAIRS{
        if args.len() != 9 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();     
        let factory_addr = args[4].clone();
        let token_0 = args[5].clone();
        let token_1 = args[6].clone();
        let token_hash = args[7].clone();
        let staking = args[8].clone(); 
        let staking_enabled =  staking.parse::<bool>().unwrap();   
        if staking_enabled == true {          
            if args.len() != 11 {
                return Err(Error::new(ErrorKind::Other, "Please provide all args"));
            } 
            let reward_addr = args[9].clone();
            let reward_addr_code_hash = args[11].clone();
            let amount = args[10].clone();
            let amount_u128 = amount.parse::<u128>().unwrap();

            add_amm_pairs_with_staking(factory_addr.clone(),&backend, &account_name, token_0.clone(),token_1.clone(),token_hash.clone(),             
            reward_addr, reward_addr_code_hash, Uint128::from(amount_u128),reports)?;
        }
        else{
            add_amm_pairs_no_staking(factory_addr.clone(),&backend, &account_name, token_0.clone(),token_1.clone(),token_hash.clone(),reports)?;
        }
      
        println!("Adding AMM Pair has to Factory {} has been completed", factory_addr.clone());        
    }
        
    if args_command == CMDLISTAMMPAIR{
        if args.len() != 5 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        }   
        let factory_add = args[2].clone();
        let start = args[3].clone();
        let limit = args[4].clone();
        let start_u64 = start.parse::<u64>().unwrap();
        let limit_u8 = limit.parse::<u8>().unwrap();
        list_pair_from_factory(factory_add.clone(), start_u64, limit_u8)?;            
    }

    if args_command == CMDBALANCE{
        if args.len() != 5 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        }   
        let snip20_addr = args[2].clone();
        let spender = args[3].clone();  
        let viewing_key = args[4].clone();     
        balance_snip20_query(snip20_addr.clone(), spender.clone(), viewing_key.clone())?;            
    }    

    if args_command == CMDADDLIQ{
        if args.len() != 11 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        }   

        let account_name = args[2].clone();
        let backend = args[3].clone();  
        let pair_addr = args[4].clone();
        let token_0 = args[5].clone();
        let token_1 = args[6].clone();
        let token_code_hash = args[7].clone();
        let amount_0 = args[8].clone();
        let amount_1 = args[9].clone();
        let staking = args[10].clone();
        let amount_0_u128 = amount_0.parse::<u128>().unwrap();
        let amount_1_u128 = amount_1.parse::<u128>().unwrap();
        let staking_bool = staking.parse::<bool>().unwrap();
        add_liquidity(&account_name, &backend,pair_addr.clone(),token_0.clone(), token_1.clone(), token_code_hash.clone(),Uint128::from(amount_0_u128),
        Uint128::from(amount_1_u128), staking_bool, reports)?;
    }
    Ok(())
}

fn print_contract_details_cli(contract: NetContract, name: String) {
    println!("{} - Contract Address {}", name, contract.address);
    println!("{} - Code Hash {}",name, contract.code_hash);
    println!("{} - Label {}", name, contract.label);
    println!("{} - Id {}", name, contract.id);
}

fn print_stored_contract_details_cli(contract: StoredContract, name: String) {    
    println!("{} - Code Hash {}",name, contract.code_hash);    
    println!("{} - Id {}", name, contract.id);
}

pub fn print_help() -> io::Result<()>
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(b"Welcome to the Shadeswap CLI.")?;
    handle.write_all(b"\n\t1. Command:: snip20 <account_name> <keyring_backend> <name> <symbol> <decimal> <viewing_key> <enable_redeem> <enable_deposit> <enable_burn> <enable_mint> <total_supply> -- Create new Snip20 Contract")?;
    handle.write_all(b"\n\t2. Command:: factory <account_name> <keyring_backend> -- Create new Factory Contract")?;
    handle.write_all(b"\n\t3. Command:: router <account_name> <keyring_backend>  <viewing_key> <pair_contract_code_hash> -- Create new Router Contract")?;
    handle.write_all(b"\n\t4. Command:: store_amm_pair <account_name> <keyring_backend> -- Store AMM Pair Contract")?;
    handle.write_all(b"\n\t5. Command:: reg_snip20 <account_name> <keyring_backend> <snip20_address> <snip20_code_hash> <router_address> -- Register Snip20 to Router")?;
    handle.write_all(b"\n\t6. Command:: allow_snip20 <account_name> <keyring_backend> <snip20_address> <spender> <amount> -- Increase Allowance for SNIP20")?;
    handle.write_all(b"\n\t7. Command:: mint_snip20 <account_name> <keyring_backend> <snip20_address> <recipient> <amount> <amount_uscrt> -- Mint Snip20")?;
    handle.write_all(b"\n\t8. Command:: store_stake <account_name> <keyring_backend> -- Store Staking Contract Contract")?;
    handle.write_all(b"\n\t9. Command:: add_amm_pair <account_name> <keyring_backend> <factory_addr> <token__addr> <token_1_addr> <token_code_hash> <staking - bool>? - if yes additional args <reward_addr> <reward_addr_code_hash> <reward_amount> -- Add new Pair for Factory")?;
    handle.write_all(b"\n\t10. Command:: list_amm_pair <factory_addr> <start> <limit> -- List All Pairs for Factory")?;
    handle.write_all(b"\n\t11. Command:: add_liq <account_name> <keyring_backend> <pair_addr> <token_0_addr> <token_1_addr> <token_code_hash> <amount_0> <amount_1> <staking - bool> -- Add Liquidity to the AMM Pair")?;
    handle.write_all(b"\n\t12. Command:: snip20_bal <snip20_addr> <spender> <viewing_key> -- Balance Snip 20 for spender")?;
    handle.write_all(b"\n\t13. Command:: set_reward_token <account_name> <keyring_backend> <staking_addr> <reward_token_addr> <reward_token_hash> <amount> <valid_to> -- Set Reward Token for Staking Contract")?;
    handle.write_all(b"\n\t14. Command:: deposit <account_name> <keyring_backend> <token_addr> <amount> -- Deposit to Snip20 Token")?;
    handle.write_all(b"\n\t15. Command:: set_viewing_key <account_name> <keyring_backend> <token_addr> <key> -- Set Viewing Key")?;
    handle.write_all(b"\n")?;
    handle.flush()?;
  
    Ok(())
}

