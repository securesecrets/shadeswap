pub mod staking_lib{

    use cosmwasm_std::{Uint128, Empty};
    use secret_multi_test::{Contract, ContractWrapper};
    use staking::contract::{execute as staking_execute, instantiate as staking_instantiate, query as staking_query};
    use shadeswap_shared::{staking::StakingContractInit, core::{TokenType, ContractInstantiationInfo}};
    
    pub fn staking_contract_store_in() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query);
        Box::new(contract)
    }

    pub fn create_staking_info_contract(
        code_id: u64,
        code_hash: &str,
        daily_reward_amount: Uint128,
        reward_token: TokenType,
        valid_to: Uint128
    ) -> StakingContractInit {
        StakingContractInit{
            contract_info: ContractInstantiationInfo{
                code_hash: code_hash.to_string(),
                id: code_id,
            },
            daily_reward_amount: daily_reward_amount,
            reward_token: reward_token,
            valid_to: valid_to,
            decimals: 18u8,
        }
    }
}