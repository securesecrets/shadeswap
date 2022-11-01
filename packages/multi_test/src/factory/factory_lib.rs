pub mod factory_lib{
    use cosmwasm_std::{StdResult, ContractInfo, Addr, Empty};
    use secret_multi_test::{App, Executor, Contract, ContractWrapper};
    use shadeswap_shared::utils::asset::Contract as SContract;
    use crate::factory::factory_mock::factory_mock::InitMsg;
    use crate::factory::factory_mock::factory_mock::{execute, instantiate, query};

    pub fn store_init_factory_contract(
        router: &mut App,
        admin: &SContract,
        sender: &str
    ) 
    -> StdResult<ContractInfo>
    {        
        let contract_info = router.store_code(factory_contract_store());   
        let contract = router.instantiate_contract(
            contract_info, 
            Addr::unchecked(sender.to_string()), 
            &InitMsg{
                admin_auth: admin.clone(),
            }, 
            &[], 
            "staking", 
            Some(sender.to_string())
        ).unwrap();
        Ok(contract)
    }

    pub fn factory_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    }


}