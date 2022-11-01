pub mod amm_pairs_lib{
    use cosmwasm_std::{ContractInfo, StdResult, Addr, to_binary, Empty};
    use secret_multi_test::{App, ContractWrapper, Executor, Contract};
    use shadeswap_shared::{core::ContractInstantiationInfo};
    use shadeswap_shared::msg::amm_pair::InitMsg;
    use crate::amm_pairs::amm_pairs_mock::amm_pairs_mock::{execute, instantiate, query};
    use crate::help_lib::integration_help_lib::{snip20_lp_token_contract_store, create_token_pair};
    use shadeswap_shared::utils::asset::Contract as SContract;
    
    pub fn store_init_amm_pair_contract(
        router: &mut App,
        token_0: &SContract,
        token_1: &SContract,
        factory: &SContract, 
        admin_auth: &SContract,
        sender: &Addr     
    ) -> StdResult<ContractInfo>
    {        
        let contract_info = router.store_code(amm_pair_contract_store());   
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
                prng_seed: to_binary("seed")?,
                entropy: to_binary("seed")?,
                admin_auth: admin_auth.clone() ,
                staking_contract: None,
                custom_fee: None,
                callback: None,
            }, 
            &[], 
            "amm_pairs", 
            Some(sender.to_string())
        ).unwrap();
        Ok(contract)
    }

    pub fn amm_pair_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    } 

}