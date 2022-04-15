
pub use shadeswap_shared::{
    fadroma::{
        scrt_addr::Canonize,
        scrt_link::{ContractLink, ContractInstantiationInfo},
        scrt::{
            from_binary,
            testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
            to_binary, Api, Binary, Env, HandleResponse, HumanAddr, Querier, StdError,
            StdResult, Storage, Uint128, Extern
        },
        scrt_storage::{load, save},
    },
    msg::factory::{ QueryResponse},
    Pagination, TokenPair, TokenType,
};


use crate::msg::InitMsg;
use crate::state::Config;

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::contract::create_pair;
    use crate::contract::init;
    use crate::state::config_read;
    use crate::state::config_write;
    pub use shadeswap_shared::{
        fadroma::{
            scrt_addr::Canonize,
            scrt_link::{ContractLink, ContractInstantiationInfo},
            scrt::{
                from_binary,
                testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
                to_binary, Api, Binary, Env, HandleResponse, HumanAddr, Querier, StdError,
                StdResult, Storage, Uint128,
            },
            scrt_storage::{load, save},
        },
        msg::factory::{ QueryResponse},
        Pagination, TokenPair, TokenType,
    };

    #[test]
    fn ok_init() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(0);
        assert!(init(deps, env, (&config).into()).is_ok());
        assert_eq!(config, config_read(deps)?);
        Ok(())
    }

    #[test]
    fn create_pair_ok() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let config = mkconfig(0);

        config_write(deps, &config)?;

        let result = create_pair(deps, mkenv("sender"));

        assert!(result.is_ok());
        Ok(())
    }

    /*
    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // not anyone can reset
        let unauth_env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env("creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }*/
}

fn mkconfig(id: u64) -> Config<HumanAddr> {
    Config::from_init_msg(InitMsg {
        pair_contract: ContractInstantiationInfo {
            id,
            code_hash: "2341586789".into(),
        },
        amm_settings: todo!(),
    })
}

fn mkdeps() -> Extern<impl Storage, impl Api, impl Querier> {
    mock_dependencies(30, &[])
}

fn mkenv(sender: impl Into<HumanAddr>) -> Env {
    mock_env(sender, &[])
}

impl Into<InitMsg> for &Config<HumanAddr> {
    fn into(self) -> InitMsg {
        InitMsg {
            pair_contract: self.pair_contract.clone(),
            amm_settings: todo!(),
        }
    }
}
