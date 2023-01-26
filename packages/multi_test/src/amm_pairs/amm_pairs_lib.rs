pub mod amm_pairs_lib{
    use cosmwasm_std::{ContractInfo, StdResult, Addr, to_binary, Empty, Uint128, Coin};
    use secret_multi_test::{App, ContractWrapper, Executor, Contract};
    use shadeswap_shared::amm_pair::{AMMSettings, AMMPair};
    use shadeswap_shared::core::{ContractInstantiationInfo, CustomFee, Fee, TokenPair, TokenType, TokenPairAmount};
    use shadeswap_shared::msg::amm_pair::{InitMsg, ExecuteMsg};
    use crate::amm_pairs::amm_pairs_mock::amm_pairs_mock::{execute, instantiate, query};
    use crate::help_lib::integration_help_lib::{snip20_lp_token_contract_store, create_token_pair};
    use shadeswap_shared::utils::asset::Contract as SContract;
    use crate::amm_pairs::amm_pairs_mock::amm_pairs_mock::reply;
    use amm_pair::contract::{execute as amm_pair_execute, instantiate as amm_pair_instantiate };
    use shadeswap_shared::staking::StakingContractInit;
    
    pub fn store_init_amm_pair_contract(       
        router: &mut App, 
        sender: &Addr, 
        token_0: &SContract, 
        token_1: &SContract, 
        factory: &SContract, 
        admin_auth: &SContract,
        store_code: Box<dyn Contract<Empty>>,
        seed: &str,
        staking_contract: Option<StakingContractInit>,
        custom_fee: Option<CustomFee>,      
    ) -> StdResult<ContractInfo>
    {             
        let contract_info = router.store_code(store_code);
        let lp_token_info =  router.store_code(snip20_lp_token_contract_store()); 
        let contract = router.instantiate_contract(
            contract_info, 
            sender.clone(), 
            &InitMsg{
                pair: create_token_pair(&token_0, &token_1),
                lp_token_contract: ContractInstantiationInfo{
                    code_hash: lp_token_info.code_hash,
                    id: lp_token_info.code_id,
                },
                factory_info: Some(factory.clone()),
                prng_seed: to_binary(seed)?,
                entropy: to_binary(seed)?,
                admin_auth: admin_auth.clone() ,
                staking_contract: staking_contract,
                custom_fee: custom_fee,
                arbitrage_contract: None,
                lp_token_decimals: 18u8,
                lp_token_custom_label: None,
            }, 
            &[], 
            "amm_pairs", 
            Some(sender.to_string())
        ).unwrap();
        Ok(contract)       
    }

    pub fn create_amm_settings(
        lp_fee_nom: u8,
        lp_fee_denom: u16,
        shade_fee_nom: u8,
        shade_fee_denom: u16,
        shade_dao_address: &Addr
    ) -> AMMSettings
    {
        AMMSettings{
            lp_fee: Fee::new(lp_fee_nom, lp_fee_denom),
            shade_dao_fee: Fee::new(shade_fee_nom, shade_fee_denom),
            shade_dao_address: SContract { address: shade_dao_address.clone(), code_hash: "".to_string() },
        }
    }

    pub fn create_amm_pairs(
        address: &Addr,
        enabled: bool,
        token_pair: TokenPair
    ) -> AMMPair{
        AMMPair { 
            pair: token_pair, 
            address: address.clone(), 
            enabled: enabled,
            code_hash: "".to_string()  }
    }

    pub fn create_native_token(denom: &str) -> TokenType{
        TokenType::NativeToken { denom: denom.to_string() }
    }

    pub fn create_custom_token(contract: &ContractInfo) -> TokenType{
        TokenType::CustomToken { 
            contract_addr: contract.address.clone(), 
            token_code_hash: contract.code_hash.clone() } 
    }

    pub fn create_token_pair_amount(
        token_pair: &TokenPair, 
        amount_0: Uint128, 
        amount_1: Uint128) -> TokenPairAmount{
        TokenPairAmount{
            pair: token_pair.clone(),
            amount_0: amount_0,
            amount_1: amount_1,
        }
    }

    pub fn add_liquidity_to_amm_pairs(
        router: &mut App,
        contract: &ContractInfo,
        pair: &TokenPair,       
        amount_0: Uint128,
        amount_1: Uint128,
        expected_return: Option<Uint128>,
        staking: Option<bool>,
        sender: &Addr,
        funds: &[Coin]
    ) -> StdResult<()>{
        let add_liq_msg = ExecuteMsg::AddLiquidityToAMMContract { 
            deposit: create_token_pair_amount(
                &pair,
                            amount_0,
                            amount_1
            ), 
            expected_return: expected_return, 
            staking: staking,
            execute_sslp_virtual_swap: None,
        };

        let _  = router.execute_contract(
            sender.to_owned(),                
            contract, 
            &add_liq_msg,
            funds
        ).unwrap();

        Ok(())
    }

    pub fn init_amm_pair(      
        router: &mut App, 
        sender: &Addr, 
        token_0: &SContract, 
        token_1: &SContract, 
        factory: &SContract, 
        admin_auth: &SContract,
        mock: bool,
        seed: &str,
        staking_contract: Option<StakingContractInit>,
        custom_fee: Option<CustomFee> 
    ) -> StdResult<ContractInfo> {
        // Create AMM_Pair or Mock
        if mock {
            return store_init_amm_pair_contract(
                router, 
                sender, 
                token_0, 
                token_1, 
                factory, 
                admin_auth,
                amm_pair_contract_store(), 
                seed,
                staking_contract,
                custom_fee
            ) 
        }
     
        return  store_init_amm_pair_contract(
            router, 
            sender, 
            token_0, 
            token_1, 
            factory, 
            admin_auth,
            amm_pair_contract_store_in(), 
            seed,
            staking_contract,
            custom_fee
        ) 
    }

    pub fn amm_pair_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    } 

    pub fn amm_pair_contract_store_in() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(amm_pair_execute, amm_pair_instantiate, query).with_reply(reply);
        Box::new(contract)
    } 

}