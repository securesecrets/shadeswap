use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg, HandleMsg, InvokeMsg, QueryMsgResponse}};
use shadeswap_shared::token_amount::{{TokenAmount}};
use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
use shadeswap_shared::token_type::{{TokenType}};
use crate::state::{Config, store_config, load_config};
use crate::state::swapdetails::{SwapInfo, SwapResult};
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery, 
            secret_toolkit::snip20,        
        },
        scrt_uint256::Uint256,
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_vk::ViewingKey,
    },
 
};
use composable_snip20::msg::{{InitMsg as Snip20ComposableMsg, InitConfig as Snip20ComposableConfig}};

#[test]
    fn init_ok() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("test");
        let config = mkinitconfig(0);
        assert!(init(deps, env, (&config).into()).is_ok());
        assert_eq!(config, config_read(deps)?);
        Ok(())
    }


    fn mkinitconfig(id: u64) -> Config<HumanAddr> {
        Config::init(InitMsg {
            pair_contract: ContractInstantiationInfo {
                id,
                code_hash: "2341586789".into(),
            },
            amm_settings: AMMSettings {
                swap_fee: Fee::new(28, 10000),
                shadeswap_fee: Fee::new(2, 10000),
                shadeswap_burner: None,
            },
        })
    }