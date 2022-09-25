
pub mod snip20_lib{
    use std::io;

    use secretcli::{secretcli::Report, cli_types::NetContract};

    use crate::utils::{InitConfig, init_snip20_cli, GAS};

    pub const SNIP20_FILE: &str = "../../compiled/snip20.wasm.gz";
    
    pub fn create_new_snip_20(account_name: &str, backend: &str, name:&str, symbol:&str, decimal: u64, 
        viewing_key:&str, reports: &mut Vec<Report>) -> io::Result<NetContract>
    {       
        let snip20 = init_snip20_contract(&name.trim(), &symbol.trim(),
        reports, decimals, account_name, backend)?;
    
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
        decimal: u8, account_name: &str, keyring_backend: &str) -> io::Result<NetContract>{
          
        let config = InitConfig{
            enable_burn: Some(true),
            enable_mint: Some(true),
            enable_deposit : Some(true),
            enable_redeem: Some(false),
            public_total_supply: Some(true),
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
        viewingKey: &str, 
        netContract: &NetContract, 
        reports: &mut Vec<Report>,
        account_name: &str,
        backend: &str) ->io::Result<()>{
        let msg = snip20_reference_impl::msg::ExecuteMsg::SetViewingKey {
            key: String::from(viewingKey),
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
}
