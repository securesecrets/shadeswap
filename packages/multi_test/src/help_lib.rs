

pub mod integration_help_lib{   
    use std::ops::Add;
    use std::time::{SystemTime, UNIX_EPOCH};
    use cosmwasm_std::CosmosMsg;
    use cosmwasm_std::Empty;
    use query_authentication::permit::Permit;
    use query_authentication::transaction::PermitSignature;
    use query_authentication::transaction::PubKey;
    use secret_multi_test::AppResponse;
    use secret_multi_test::next_block;
    use shadeswap_shared::staking;
    use shadeswap_shared::utils::testing::TestingExt;    
    use cosmwasm_std::{Addr, ContractInfo, StdResult, Uint128, Coin, Binary, WasmMsg};
    use secret_multi_test::Contract;
    use secret_multi_test::ContractWrapper;
    use secret_multi_test::{App, Executor};    
    use shadeswap_shared::{
        msg::staking::{InitMsg, InvokeMsg}, 
        core::TokenPair, 
        core::{TokenType, ContractLink}, 
        snip20::{InitConfig, InstantiateMsg, self}, 
        query_auth::PermitData, 
        staking::QueryData
    };
    use snip20_reference_impl::contract::{execute as snip20_execute, instantiate as snip20_instantiate, query as  snip20_query};
    use cosmwasm_std::to_binary;  
    use crate::auth_query::auth_query::{{execute as auth_execute, instantiate as auth_instantiate, query as auth_query, InitMsg as AuthInitMsg}};
    use crate::util_addr::util_addr::{OWNER, TOKEN_B, TOKEN_A};
    use crate::factory_mock::factory_mock::{execute as factory_execute, query as factory_query,instantiate as factory_instantiate, InitMsg as FactoryInitMsg };
    use crate::staking::staking_mock::staking_mock::{execute as staking_execute, query as staking_query,instantiate as staking_instantiate, InitMsg as StakingInitMsg };
    type TestPermit = Permit<PermitData>;
    
    pub fn mk_token_pair() -> TokenPair{
        return TokenPair(
            TokenType::CustomToken { contract_addr: mk_address(TOKEN_A), token_code_hash: "".to_string() },
            TokenType::CustomToken { contract_addr: mk_address(TOKEN_B), token_code_hash: "".to_string() }
        );
    }

    pub fn store_init_auth_contract(router: &mut App) 
    -> StdResult<ContractInfo>
    {        
        let auth_contract_info = router.store_code(auth_permit_contract_store());   
        let auth_contract = router.instantiate_contract(
            auth_contract_info, 
            mk_address(&OWNER).to_owned(), 
            &AuthInitMsg{}, 
            &[], 
            "auth_permit", 
            Some(OWNER.to_string())
        ).unwrap();
        Ok(auth_contract)
    }

    
    pub fn store_init_factory_contract(router: &mut App) 
    -> StdResult<ContractInfo>
    {        
        let auth_contract_info = router.store_code(factory_contract_store());   
        let auth_contract = router.instantiate_contract(
            auth_contract_info, 
            mk_address(&OWNER).to_owned(), 
            &FactoryInitMsg{}, 
            &[], 
            "staking", 
            Some(OWNER.to_string())
        ).unwrap();
        Ok(auth_contract)
    }

    pub fn store_init_staking_contract(router: &mut App) 
    -> StdResult<ContractInfo>
    {        
        let auth_contract_info = router.store_code(auth_permit_contract_store());   
        let auth_contract = router.instantiate_contract(
            auth_contract_info, 
            mk_address(&OWNER).to_owned(), 
            &StakingInitMsg{}, 
            &[], 
            "staking", 
            Some(OWNER.to_string())
        ).unwrap();
        Ok(auth_contract)
    }


    pub fn roll_blockchain(router: &mut App, count: u128) -> StdResult<()>{
        for i in 1..count {            
            router.update_block(next_block);         
        }
        Ok(())
    }
    
    pub fn snip20_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(snip20_execute, snip20_instantiate, snip20_query);
        Box::new(contract)
    } 

    pub fn auth_permit_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(auth_execute, auth_instantiate, auth_query);
        Box::new(contract)
    }

    pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query);
        Box::new(contract)
    }

    pub fn factory_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(factory_execute, factory_instantiate, factory_query);
        Box::new(contract)
    }

    pub fn mk_address(address: &str) -> Addr{
        return Addr::unchecked(address.to_string())
    }

    pub fn mk_contract_link(address: &str) -> ContractLink{
        return ContractLink{
            address: mk_address(address),
            code_hash: "".to_string(),
        }       
    }

    pub fn get_current_timestamp() -> StdResult<Uint128> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Ok(Uint128::from(since_the_epoch.as_millis()))
    }

    pub fn get_snip20_balance(
        router: &mut App, 
        contract: &ContractInfo, 
        from: &str, 
        view_key: &str
    ) -> Uint128 {
        let msg = to_binary(&snip20::QueryMsg::Balance {
            address: from.to_string(),
            key: view_key.to_string(),
        }).unwrap();
    
        let balance: snip20::QueryAnswer = router.query_test(contract.to_owned(), msg).unwrap();
    
        if let snip20::QueryAnswer::Balance { amount } = balance {
            return amount;
        }
        Uint128::zero()
    }

  
    pub fn mk_create_permit_data(pub_key: &str, sign: &str, chain_id: &str) 
    -> StdResult<TestPermit>
    {
        //secretd tx sign-doc file --from a
        let newPermit = TestPermit{
            params: PermitData { data: to_binary(&QueryData {}).unwrap(), key: "0".to_string()},
            chain_id: Some(chain_id.to_string()),
            sequence: Some(Uint128::zero()),
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(pub_key).unwrap()),
                signature: Binary::from_base64(sign).unwrap(),
            },
            account_number: Some(Uint128::zero()),
            memo: Some("".to_string())
        };
        return Ok(newPermit);
    }

    pub fn get_current_block_time(router: &App) -> Uint128 {
        let current_timestamp = router.block_info().time;
        Uint128::new(current_timestamp.seconds() as u128)
    }

    pub fn mint_deposit_snip20(
        router: &mut App, 
        contract: &ContractInfo, 
        recipient: &Addr,
        amount: Uint128,
        sender: &Addr
    ) {
        set_viewing_key(router, &contract, "password", sender).unwrap();
        deposit_snip20(router,&contract, amount, &sender).unwrap();
        mint_snip20(router, amount, &recipient,&contract, &sender).unwrap();       
    }

    pub fn increase_allowance(
        router: &mut App, 
        contract: &ContractInfo,
        amount: Uint128,
        spender: &Addr,
        sender: &Addr
    ) 
    -> StdResult<AppResponse>{
        let msg = snip20::ExecuteMsg::IncreaseAllowance { 
            spender: spender.to_string() , 
            amount: amount, 
            expiration: None, 
            padding: None 
        };

        let respone = router.execute_contract(sender.to_owned(), contract, &msg, &[]).unwrap();        
        Ok(respone)
    }

 
    pub fn send_snip20_to_stake(
        router: &mut App, 
        contract: &ContractInfo,
        stake_contract: &ContractInfo,
        amount: Uint128,
        staker: &Addr,
        sender: &Addr
    ) -> StdResult<AppResponse>{        
        let invoke_msg = to_binary(&InvokeMsg::Stake {
            from: staker.to_owned(),
        })?;
       
        let msg = snip20_reference_impl::msg::ExecuteMsg::Send {
            recipient: stake_contract.address.to_owned(),
            recipient_code_hash: Some(stake_contract.code_hash.clone()),
            amount: amount,
            msg: Some(invoke_msg),
            memo: None,
            padding: None,
        };

        let response: AppResponse = router.execute_contract(
            sender.to_owned(),
            &contract.clone(),
            &msg,
            &[], // 
        )
        .unwrap();               
        Ok(response)
    }

    pub fn snip20_send(
        router: &mut App, 
        contract: &ContractInfo,
        recipient: &Addr,
        amount: Uint128,
        sender: &Addr
    ) -> StdResult<AppResponse>{       
        let msg = snip20_reference_impl::msg::ExecuteMsg::Send {
            recipient: recipient.to_owned(),
            recipient_code_hash: None,
            amount: amount,
            msg: None,
            memo: None,
            padding: None,
        };

        let response: AppResponse = router.execute_contract(
            sender.to_owned(),
            &contract.clone(),
            &msg,
            &[], 
        )
        .unwrap();               
        Ok(response)
    }

    pub fn mint_snip20(
        router: &mut App, 
        amount: Uint128,
        recipient: &Addr,
        contract: &ContractInfo,
        sender: &Addr
    ) -> StdResult<AppResponse>{
        let msg = snip20_reference_impl::msg::ExecuteMsg::Mint { 
            recipient: recipient.to_owned(), 
            amount: amount, 
            memo: None, 
            padding: None 
        };

        let response = router.execute_contract(
            sender.to_owned(),
            contract,
            &msg,
            &[Coin{ denom: "uscrt".to_string(), amount: amount}], // Coin{ denom: "uscrt".to_string(), amount: Uint128::new(3000000)}
        )
        .unwrap();
        Ok(response)
    }

    pub fn print_events(app_response: AppResponse) -> (){
        for i in app_response.events {
            println!("{:}", i.ty);
            for msg in i.attributes{
                println!("key {:} - value {:}", msg.key, msg.value)
            }
        }
    }

    pub fn deposit_snip20(
        router: &mut App, 
        contract: &ContractInfo,
        amount: Uint128,
        sender: &Addr
    ) -> StdResult<AppResponse>{
        let msg = snip20::ExecuteMsg::Deposit { padding: None };
        let app_response = router.execute_contract(
            sender.clone(),
            contract,
            &msg,
            &[Coin{ 
                denom: "uscrt".to_string(), 
                amount: amount
            }], 
        )
        .unwrap();
      
        Ok(app_response)
    }

    pub fn set_viewing_key(
        router: &mut App,
        contract: &ContractInfo,
        key: &str,
        sender: &Addr
    ) -> StdResult<AppResponse>{
        let msg = snip20::ExecuteMsg::SetViewingKey { key: key.to_string(), padding: None};
        let app_response = router.execute_contract(
            sender.to_owned(),
            contract,
            &msg,
            &[], 
        )
        .unwrap();
        Ok(app_response)
    }

      pub fn generate_snip20_contract(
        router: &mut App, 
        name: String, 
        symbol: String, 
        decimal: u8) -> StdResult<ContractInfo> {

        let snip20_contract_code_id = router.store_code(snip20_contract_store());        
        let init_snip20_msg = InstantiateMsg {
            name: name.to_string(),
            admin: Some(OWNER.to_string()),
            symbol: symbol.to_string(),
            decimals: decimal,
            initial_balances: Some(vec![snip20::InitialBalance {
                address: OWNER.into(),
                amount: Uint128::from(1000000000000000u128),
            }]),
            prng_seed: to_binary("password")?,
            config: Some(InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(true),
                enable_redeem: Some(false),
                enable_mint: Some(true),
                enable_burn: Some(true),
                enable_transfer: Some(true),
            }),
            query_auth: None,
        };
        let init_snip20_code_id = router
            .instantiate_contract(
                snip20_contract_code_id,
                mk_address(&OWNER),
                &init_snip20_msg,
                &[],
                "token_a",
                Some(OWNER.to_string()),
            ).unwrap();
        Ok(init_snip20_code_id)
    }


}