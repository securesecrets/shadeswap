use cosmwasm_std::{
    from_binary, Api, Binary, Env, Response, Querier, StdError,
    StdResult, Storage,
};
use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::core::ContractInstantiationInfo;
use crate::core::ContractLink;

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

pub mod router {
    use super::{amm_pair::SwapResult, *};
    use crate::core::{ViewingKey, ContractLink, TokenAmount};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub enum InvokeMsg {
        SwapTokensForExact {
            paths: Vec<String>,
            expected_return: Option<Uint128>,
            recipient: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub factory_address: ContractLink,
        pub prng_seed: Binary,
        pub entropy: Binary,
        pub viewing_key: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        // SNIP20 receiver interface
        Receive {
            from: String,
            msg: Option<Binary>,
            amount: Uint128,
        },
        SwapTokensForExact {
            /// The token type to swap from.
            offer: TokenAmount,
            expected_return: Option<Uint128>,
            path: Vec<String>,
            recipient: Option<String>,
        },
        SwapCallBack {
            last_token_out: TokenAmount,
            signature: Binary,
        },
        RegisterSNIP20Token {
            token: String,
            token_code_hash: String,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        SwapSimulation {
            offer: TokenAmount,
            path: Vec<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        SwapSimulation {
            total_fee_amount: Uint128,
            lp_fee_amount: Uint128,
            shade_dao_fee_amount: Uint128,
            result: SwapResult,
            price: String,
        },
    }
}

pub mod amm_pair {
    use super::*;
    use crate::{
        amm_pair::AMMSettings,
        stake_contract::StakingContractInit,
        Pagination, core::{ContractLink, ContractInstantiationInfo, Callback, TokenPairAmount, TokenAmount, CustomFee, Fee, TokenPair},
    };
    use cosmwasm_std::{Decimal, Addr};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, JsonSchema)]
    pub struct SwapInfo {
        pub total_fee_amount: Uint128,
        pub lp_fee_amount: Uint128,
        pub shade_dao_fee_amount: Uint128,
        pub result: SwapResult,
        pub price: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug, JsonSchema)]
    pub struct SwapResult {
        pub return_amount: Uint128,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
    pub struct TradeHistory {
        pub price: String,
        pub amount_out: Uint128,
        pub amount_in: Uint128,
        pub timestamp: u64,
        pub direction: String,
        pub total_fee_amount: Uint128,
        pub lp_fee_amount: Uint128,
        pub shade_dao_fee_amount: Uint128,
        pub height: u64,
        pub trader: String,
    }
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub pair: TokenPair,
        pub lp_token_contract: ContractInstantiationInfo,
        pub factory_info: ContractLink,
        pub prng_seed: Binary,
        pub entropy: Binary,
        pub admin: Option<Addr>,
        pub staking_contract: Option<StakingContractInit>,
        pub custom_fee: Option<CustomFee>,
    }
    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        AddLiquidityToAMMContract {
            deposit: TokenPairAmount,
            slippage: Option<Decimal>,
            staking: Option<bool>,
        },
        SwapTokens {
            /// The token type to swap from.
            offer: TokenAmount,
            expected_return: Option<Uint128>,
            to: Option<String>,
            router_link: Option<ContractLink>,
            callback_signature: Option<Binary>,
        },
        // SNIP20 receiver interface
        Receive {
            from: String,
            msg: Option<Binary>,
            amount: Uint128,
        },
        // Sent by the LP token contract so that we can record its address.
        OnLpTokenInitAddr,
        AddWhiteListAddress {
            address: String,
        },
        RemoveWhitelistAddresses {
            addresses: Vec<String>,
        },
        SetAMMPairAdmin {
            admin: String,
        },
        SetStakingContract {
            contract: ContractLink,
        },
        SetCustomPairFee {
            shade_dao_fee: Fee,
            lp_fee: Fee,
        },
    }
    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum InvokeMsg {
        SwapTokens {
            expected_return: Option<Uint128>,
            to: Option<String>,
            router_link: Option<ContractLink>,
            callback_signature: Option<Binary>,
        },
        RemoveLiquidity {
            from: Option<String>,
        },
    }
    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        GetPairInfo {},
        GetTradeHistory {
            pagination: Pagination,
        },
        GetWhiteListAddress {},
        GetTradeCount {},
        GetAdmin {},
        GetStakingContract {},
        GetEstimatedPrice {
            offer: TokenAmount,
            exclude_fee: Option<bool>,
        },
        SwapSimulation {
            offer: TokenAmount,
        },
        GetShadeDaoInfo {},
        GetEstimatedLiquidity {
            deposit: TokenPairAmount,
            slippage: Option<Decimal>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        GetPairInfo {
            liquidity_token: ContractLink,
            factory: ContractLink,
            pair: TokenPair,
            amount_0: Uint128,
            amount_1: Uint128,
            total_liquidity: Uint128,
            contract_version: u32,
        },
        GetTradeHistory {
            data: Vec<TradeHistory>,
        },
        GetWhiteListAddress {
            addresses: Vec<String>,
        },
        GetTradeCount {
            count: u64,
        },
        GetAdminAddress {
            address: String,
        },
        GetClaimReward {
            amount: Uint128,
        },
        StakingContractInfo {
            staking_contract: ContractLink,
        },
        EstimatedPrice {
            estimated_price: String,
        },
        SwapSimulation {
            total_fee_amount: Uint128,
            lp_fee_amount: Uint128,
            shade_dao_fee_amount: Uint128,
            result: SwapResult,
            price: String,
        },
        ShadeDAOInfo {
            shade_dao_address: String,
            shade_dao_fee: Fee,
            lp_fee: Fee,
            admin_address: String,
        },
        EstimatedLiquidity {
            lp_token: Uint128,
            total_lp_token: Uint128,
        },
    }
}

pub mod factory {
    use super::*;
    use crate::amm_pair::AMMPair;
    use crate::core::TokenPair;
    use crate::stake_contract::StakingContractInit;
    use crate::{amm_pair::AMMSettings, Pagination};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub pair_contract: ContractInstantiationInfo,
        pub amm_settings: AMMSettings,
        pub lp_token_contract: ContractInstantiationInfo,
        pub prng_seed: Binary
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        SetConfig {
            pair_contract: Option<ContractInstantiationInfo>,
            lp_token_contract: Option<ContractInstantiationInfo>,
            amm_settings: Option<AMMSettings>,
        },
        CreateAMMPair {
            pair: TokenPair,
            entropy: Binary,
            staking_contract: Option<StakingContractInit>,
        },
        AddAMMPairs {
            amm_pairs: Vec<AMMPair>,
        },
        SetFactoryAdmin {
            admin: String,
        }
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        ListAMMPairs {
            amm_pairs: Vec<AMMPair>,
        },
        GetConfig {
            pair_contract: ContractInstantiationInfo,
            amm_settings: AMMSettings,
            lp_token_contract: ContractInstantiationInfo,
        },
        GetAMMPairAddress {
            address: String,
        },
        GetAMMSettings {
            settings: AMMSettings,
        },
        GetAdminAddress {
            address: String,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        // GetCount returns the current count as a json-encoded number
        ListAMMPairs { pagination: Pagination },
        GetAMMPairAddress { pair: TokenPair },
        GetAMMSettings,
        GetConfig,
        GetAdmin,
    }
}

pub mod staking {
    use crate::core::TokenType;

    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub staking_amount: Uint128,
        pub reward_token: TokenType, 
        pub pair_contract: ContractLink,
        pub prng_seed: Binary
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        ClaimRewards {},
        Unstake {
            amount: Uint128,
            remove_liqudity: Option<bool>,
        },
        SetLPToken {
            lp_token: ContractLink,
        },
        Receive {
            from: String,
            msg: Option<Binary>,
            amount: Uint128,
        }, 
        SetVKForStaker{
            key: String
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum InvokeMsg {
        Stake { from: String, amount: Uint128 },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {        
        GetClaimReward {staker: String, key: String, time: Uint128},
        GetContractOwner {},
        GetStakerLpTokenInfo{key: String, staker: String},
        GetRewardTokenBalance {key: String, address: String},
        GetStakerRewardTokenBalance {key: String, staker: String},
        GetConfig{}
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        ClaimReward {
            amount: Uint128,
            reward_token: ContractLink<HumanAddr>
        },
        ContractOwner {
            address: String,
        },
        StakerLpTokenInfo {
            staked_lp_token: Uint128,
            total_staked_lp_token: Uint128,
        },
        RewardTokenBalance {
            amount: Uint128,
            reward_token: ContractLink<HumanAddr>
        },
        StakerRewardTokenBalance {
            reward_amount: Uint128,
            total_reward_liquidity: Uint128,
            reward_token: ContractLink<HumanAddr>
        },
        Config{
            reward_token: ContractLink<HumanAddr>,
            lp_token: ContractLink<HumanAddr>,
            daily_reward_amount: Uint128,
            contract_owner: HumanAddr
        }
    }
}
