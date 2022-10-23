

pub mod integration_help_lib{   
    use std::time::{SystemTime, UNIX_EPOCH};
    use cosmwasm_std::CosmosMsg;
    use cosmwasm_std::Empty;
    use query_authentication::permit::Permit;
    use query_authentication::transaction::PermitSignature;
    use query_authentication::transaction::PubKey;
    use secret_multi_test::AppResponse;
    use secret_multi_test::next_block;
    use shadeswap_shared::utils::testing::TestingExt;
    use snip20_reference_impl::contract::instantiate;
    use cosmwasm_std::{Addr, ContractInfo, StdResult, Uint128, Coin, Binary, WasmMsg};
    use secret_multi_test::Contract;
    use secret_multi_test::ContractWrapper;
    use secret_multi_test::{App, Executor};
    use shadeswap_shared::staking::ExecuteMsg;
    use shadeswap_shared::{msg::staking::{InitMsg, InvokeMsg}, core::TokenPair, core::{TokenType, ContractLink}, snip20::{InitConfig, InstantiateMsg, self}, query_auth::PermitData, staking::QueryData};
    use snip20_reference_impl::contract::{execute as snip20_execute, instantiate as snip20_instantiate, query as  snip20_query};
    use cosmwasm_std::to_binary;  
    use crate::auth_query::auth_query::{{execute as auth_execute, instantiate as auth_instantiate, query as auth_query, InitMsg as AuthInitMsg}};
    use crate::util_addr::util_addr::{OWNER, TOKEN_B, TOKEN_A};
          
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
        from: String, 
        view_key: String
    ) -> Uint128 {
        let msg = to_binary(&snip20::QueryMsg::Balance {
            address: from,
            key: view_key,
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

 
    pub fn send_snip20_with_msg(
        router: &mut App, 
        contract: &ContractInfo,
        stake_contract: &ContractInfo,
        amount: Uint128,
        staker: &Addr
    ) -> StdResult<()>{
        let OWNER_ADDRESS: Addr = Addr::unchecked(OWNER);
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
            Addr::unchecked(OWNER.to_owned()),
            &contract.clone(),
            &msg,
            &[], // 
        )
        .unwrap();

        print_events(response);        
        Ok(())
    }

    pub fn snip20_send(
        router: &mut App, 
        contract: &ContractInfo,
        recipient: &ContractInfo,
        amount: Uint128,
        staker: &Addr
    ) -> StdResult<()>{       
        let msg = snip20_reference_impl::msg::ExecuteMsg::Send {
            recipient: recipient.address.to_owned(),
            recipient_code_hash: Some(recipient.code_hash.clone()),
            amount: amount,
            msg: None,
            memo: None,
            padding: None,
        };

        let response: AppResponse = router.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            &contract.clone(),
            &msg,
            &[], // 
        )
        .unwrap();

        print_events(response);        
        Ok(())
    }

    pub fn mint_snip20(
        router: &mut App, 
        amount: Uint128,
        recipient: Addr,
        contract: ContractInfo
    ) -> StdResult<()>{
        let msg = snip20_reference_impl::msg::ExecuteMsg::Mint { 
            recipient: recipient, 
            amount: amount, 
            memo: None, 
            padding: None 
        };

        let _ = router.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            &contract.clone(),
            &msg,
            &[Coin{ denom: "uscrt".to_string(), amount: amount}], // Coin{ denom: "uscrt".to_string(), amount: Uint128::new(3000000)}
        )
        .unwrap();

        Ok(())
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
        contract: ContractInfo,
        amount: Uint128
    ) -> StdResult<()>{
        let msg = snip20::ExecuteMsg::Deposit { padding: None };
        let app_response = router.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            &contract.clone(),
            &msg,
            &[Coin{ 
                denom: "uscrt".to_string(), 
                amount: amount
            }], 
        )
        .unwrap();

        for res in app_response.events.to_owned(){
            println!("{:}", res.ty);
            for msg in res.attributes{
                println!("key {:} - value {:}", msg.key, msg.value)
            }
        }
        Ok(())
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
                amount: Uint128::from(100000000u128),
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
                mk_address(&OWNER).to_owned(),
                &init_snip20_msg,
                &[],
                "token_a",
                Some(OWNER.to_string()),
            ).unwrap();
        Ok(init_snip20_code_id)
    }


}