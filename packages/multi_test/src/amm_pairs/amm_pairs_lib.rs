pub mod amm_pairs_lib{
    use cosmwasm_std::{ContractInfo, StdResult, Addr, to_binary, Empty};
    use secret_multi_test::{App, ContractWrapper, Executor, Contract};
    use shadeswap_shared::core::{ContractInstantiationInfo, CustomFee};
    use shadeswap_shared::msg::amm_pair::InitMsg;
    use crate::amm_pairs::amm_pairs_mock::amm_pairs_mock::{execute, instantiate, query};
    use crate::help_lib::integration_help_lib::{snip20_lp_token_contract_store, create_token_pair};
    use shadeswap_shared::utils::asset::Contract as SContract;
    use amm_pair::contract::{execute as amm_pair_execute, instantiate as amm_pair_instantiate, query as amm_pair_query };
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
        custom_fee: Option<CustomFee>      
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
                factory_info: factory.clone(),
                prng_seed: to_binary(seed)?,
                entropy: to_binary(seed)?,
                admin_auth: admin_auth.clone() ,
                staking_contract: staking_contract,
                custom_fee: custom_fee,
                callback: None,
            }, 
            &[], 
            "amm_pairs", 
            Some(sender.to_string())
        ).unwrap();
        Ok(contract)       
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
    ) -> Result<ContractInfo, cosmwasm_std::StdError> {
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
                custom_fee) 
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
        let contract = ContractWrapper::new_with_empty(amm_pair_execute, amm_pair_instantiate, amm_pair_query);
        Box::new(contract)
    } 

}