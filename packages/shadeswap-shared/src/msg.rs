use crate::core::ContractInstantiationInfo;
use cosmwasm_std::Binary;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
use crate::{core::{TokenAmount, TokenType}, Contract, utils::ExecuteCallback};

pub mod router {
    use cosmwasm_std::Addr;

    use super::{amm_pair::SwapResult, *};
    use crate::{core::{TokenAmount, TokenType}, Contract};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub enum InvokeMsg {
        SwapTokensForExact {
            path: Vec<Hop>,
            expected_return: Option<Uint128>,
            recipient: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub prng_seed: Binary,
        pub entropy: Binary,
        pub admin_auth: Contract
    }


    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Hop {
        pub addr: String,
        pub code_hash: String
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
            path: Vec<Hop>,
            recipient: Option<String>,
        },
        RegisterSNIP20Token {
            token_addr: String,
            token_code_hash: String,
        },
        RecoverFunds {
            token: TokenType,
            amount: Uint128,
            to: String,
            msg: Option<Binary>,
        },
    }

    impl ExecuteCallback for ExecuteMsg {
        const BLOCK_SIZE: usize = 256;
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        SwapSimulation { offer: TokenAmount, path: Vec<Hop> },
        GetConfig {},
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
        GetConfig {
        }
    }
}

pub mod amm_pair {
    use super::*;
    use crate::{
        core::{
            Callback, ContractInstantiationInfo, CustomFee, Fee, TokenAmount,
            TokenPair, TokenPairAmount, TokenType,
        },
        Pagination, staking::StakingContractInit, Contract,
    };
    use cosmwasm_std::{Addr};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    
    /// Represents the address of an exchange and the pair that it manages
    #[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
    pub struct AMMPair {
        /// The pair that the contract manages.
        pub pair: TokenPair,
        /// Address of the contract that manages the exchange.
        pub address: Addr,
        /// Used to enable or disable the AMMPair
        pub enabled: bool
    }
    
    
    #[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug,Clone)]
    pub struct AMMSettings {
        pub lp_fee: Fee,
        pub shade_dao_fee: Fee,
        pub shade_dao_address: Contract
    }
    
    pub fn generate_pair_key(pair: &TokenPair) -> Vec<u8> {
        let mut bytes: Vec<&[u8]> = Vec::new();
    
        match &pair.0 {
            TokenType::NativeToken { denom } => bytes.push(denom.as_bytes()),
            TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_bytes())
        }
    
        match &pair.1 {
            TokenType::NativeToken { denom } => bytes.push(denom.as_bytes()),
            TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_bytes())
        }
    
        bytes.sort();
    
        bytes.concat()
    }

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
    }
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub pair: TokenPair,
        pub lp_token_contract: ContractInstantiationInfo,
        pub factory_info: Contract,
        pub prng_seed: Binary,
        pub entropy: Binary,
        pub admin_auth: Contract,
        pub staking_contract: Option<StakingContractInit>,
        pub custom_fee: Option<CustomFee>,
        pub callback: Option<Callback>,
    }
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        AddLiquidityToAMMContract {
            deposit: TokenPairAmount,
            expected_return: Option<Uint128>,
            staking: Option<bool>,
        },
        SwapTokens {
            /// The token type to swap from.
            offer: TokenAmount,
            expected_return: Option<Uint128>,
            to: Option<String>
        },
        // SNIP20 receiver interface
        Receive {
            from: String,
            msg: Option<Binary>,
            amount: Uint128,
        },
        AddWhiteListAddress {
            address: String,
        },
        RemoveWhitelistAddresses {
            addresses: Vec<String>,
        },
        SetConfig {
            admin_auth: Option<Contract>
        },
        SetCustomPairFee {
            custom_fee: Option<CustomFee>,
        },
        SetViewingKey {
            viewing_key: String,
        },
        RecoverFunds {
            token: TokenType,
            amount: Uint128,
            to: String,
            msg: Option<Binary>,
        },
    }

    impl ExecuteCallback for ExecuteMsg {
        const BLOCK_SIZE: usize = 256;
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum InvokeMsg {
        SwapTokens {
            expected_return: Option<Uint128>,
            to: Option<String>,
        },
        RemoveLiquidity {
            from: Option<String>,
        },
    }
    #[derive(Serialize, Deserialize, JsonSchema,  Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        GetConfig {},
        GetPairInfo {},
        GetTradeHistory {
            api_key: String,
            pagination: Pagination,
        },
        GetWhiteListAddress {},
        GetTradeCount {},
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
            deposit: TokenPairAmount
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        GetPairInfo {
            liquidity_token: Contract,
            factory: Contract,
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
            addresses: Vec<Addr>,
        },
        GetTradeCount {
            count: u64,
        },
        GetClaimReward {
            amount: Uint128,
        },
        StakingContractInfo {
            staking_contract: Option<Contract>,
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
            admin_auth: Contract,
        },
        EstimatedLiquidity {
            lp_token: Uint128,
            total_lp_token: Uint128,
        },
        GetConfig {
            factory_contract: Contract,
            lp_token: Contract,
            staking_contract: Option<Contract>,
            pair: TokenPair,
            custom_fee: Option<CustomFee>,
        },
    }
}

pub mod factory {
    use super::*;
    use crate::amm_pair::AMMPair;
    use crate::core::TokenPair;
    use crate::Contract;
    use crate::staking::StakingContractInit;
    use crate::{amm_pair::AMMSettings, Pagination};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub pair_contract: ContractInstantiationInfo,
        pub amm_settings: AMMSettings,
        pub lp_token_contract: ContractInstantiationInfo,
        pub prng_seed: Binary,
        pub api_key: String,
        //Set the default authenticator for all permits on the contracts
        pub authenticator: Option<Contract>,
        pub admin_auth: Contract
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        SetConfig {
            pair_contract: Option<ContractInstantiationInfo>,
            lp_token_contract: Option<ContractInstantiationInfo>,
            amm_settings: Option<AMMSettings>,
            api_key: Option<String>,
            admin_auth: Option<Contract>,
        },
        CreateAMMPair {
            pair: TokenPair,
            entropy: Binary,
            staking_contract: Option<StakingContractInit>,
        },
        AddAMMPairs {
            amm_pairs: Vec<AMMPair>,
        },
        RegisterAMMPair {
            pair: TokenPair,
            signature: Binary,
        },
    }

    impl ExecuteCallback for ExecuteMsg {
        const BLOCK_SIZE: usize = 256;
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
            authenticator: Option<Contract>,
            admin_auth: Contract,
        },
        GetAMMPairAddress {
            address: String,
        },
        AuthorizeApiKey {
            authorized: bool,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        // GetCount returns the current count as a json-encoded number
        ListAMMPairs { pagination: Pagination },
        GetAMMPairAddress { pair: TokenPair },
        GetConfig,
        AuthorizeApiKey { api_key: String },
    }
}

pub mod staking {
    use crate::{core::TokenType, query_auth::QueryPermit,Contract, stake_contract::{ClaimableInfo, RewardTokenInfo}};

    use super::*;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::Addr;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
    pub struct StakingContractInit {
        pub contract_info: ContractInstantiationInfo,
        pub daily_reward_amount: Uint128,
        pub reward_token: TokenType,
        pub valid_to: Uint128
    }

    #[cw_serde]
    pub struct QueryData {}

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub daily_reward_amount: Uint128,
        pub reward_token: TokenType,
        pub valid_to: Uint128,
        pub pair_contract: Contract,
        pub prng_seed: Binary,
        pub lp_token: Contract,
        //Used for permits
        pub authenticator: Option<Contract>,
        pub admin_auth: Contract,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        ClaimRewards {},
        ProxyUnstake {
            for_addr: String,
            amount: Uint128,
        },
        Unstake {
            amount: Uint128,
            remove_liqudity: Option<bool>,
        },
        Receive {
            from: String,
            msg: Option<Binary>,
            amount: Uint128,
        },
        SetRewardToken {
            reward_token: Contract,
            daily_reward_amount: Uint128,
            valid_to: Uint128
        },
        SetAuthenticator {
            authenticator: Option<Contract>,
        },
        SetConfig {
            admin_auth: Option<Contract>,
        },
        RecoverFunds {
            token: TokenType,
            amount: Uint128,
            to: String,
            msg: Option<Binary>,
        },        
    }

    impl ExecuteCallback for ExecuteMsg {
        const BLOCK_SIZE: usize = 256;
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum InvokeMsg {
        Stake { from: String },
        ProxyStake {
            for_addr: String
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone,JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        GetContractOwner {},
        GetConfig {},
        WithPermit {
            permit: QueryPermit,
            query: AuthQuery,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum AuthQuery {
        GetStakerLpTokenInfo {},
        GetClaimReward { time: Uint128 },
        GetRewardTokens {}
    }

    #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        ClaimRewards {
            claimable_rewards: Vec<ClaimableInfo>
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
            reward_token: Contract,
        },
        StakerRewardTokenBalance {
            reward_amount: Uint128,
            total_reward_liquidity: Uint128,
            reward_token: Contract,
        },
        Config {
            reward_token: Contract,
            lp_token: Contract,
            daily_reward_amount: Uint128,
            amm_pair: String,
            admin_auth: Contract
        },
        RewardTokens{
            tokens: Vec<RewardTokenInfo>
        }
    }
}

pub mod lp_token {
    use cosmwasm_std::Addr;

    use crate::{snip20::InitialBalance};

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    pub struct InitConfig {
        /// Indicates whether the total supply is public or should be kept secret.
        /// default: False
        pub public_total_supply: Option<bool>,
        /// Indicates whether deposit functionality should be enabled
        /// default: False
        pub enable_deposit: Option<bool>,
        /// Indicates whether redeem functionality should be enabled
        /// default: False
        pub enable_redeem: Option<bool>,
        /// Indicates whether mint functionality should be enabled
        /// default: False
        pub enable_mint: Option<bool>,
        /// Indicates whether burn functionality should be enabled
        /// default: False
        pub enable_burn: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    pub struct InstantiateMsg {
        pub name: String,
        pub admin: Option<String>,
        pub symbol: String,
        pub decimals: u8,
        pub initial_balances: Option<Vec<InitialBalance>>,
        pub prng_seed: Binary,
        pub config: Option<InitConfig>,
    }
}
