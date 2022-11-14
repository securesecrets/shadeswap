pub mod admin_help{
    use cosmwasm_std::{Empty, Addr, ContractInfo, StdResult};
    use secret_multi_test::{Contract, ContractWrapper, App, Executor};
    use crate::admin::admin_mock::admin_mock::{execute, query,instantiate, InitMsg};

    pub fn admin_contract_init_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute,instantiate, query);
        Box::new(contract)
    }

    pub fn init_admin_contract(
        router: &mut App,
        sender: &Addr
    ) -> StdResult<ContractInfo> {

        // ADMIN INIT MSG
        let init_msg = InitMsg{

        };

        let admin_store_info = router.store_code(admin_contract_init_store());
        let admin_contract = router.instantiate_contract(
            admin_store_info,
            sender.clone(),
            &init_msg,
            &[],
            "admin",
            Some(sender.to_string()),
        ).unwrap();
    Ok(admin_contract)
    }
} 