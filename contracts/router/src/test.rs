#[cfg(test)]
pub mod tests {
    use crate::state::config_read;
use crate::contract::init;
use cosmwasm_std::Extern;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::Env;
use cosmwasm_std::Querier;
use cosmwasm_std::Api;
use cosmwasm_std::Storage;
use crate::state::State;
use cosmwasm_std::StdResult;
use crate::msg::InitMsg;
use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn ok_init() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(0);
        assert!(init(deps, env, (&config).into()).is_ok());
        assert_eq!(config, config_read(&deps.storage).load()?);
        Ok(())
    }

    #[test]
    fn swap_exact_tokens_for_tokens() -> StdResult<()> {
        Ok(())
    }

    //*** */
    #[test]
    fn swap_tokens_for_exact_tokens() -> StdResult<()> {
        Ok(())
    }

    fn mkconfig(id: u64) -> State {
        State::from_init_msg(InitMsg {
        })
    }
    
    fn mkdeps() -> Extern<impl Storage, impl Api, impl Querier> {
        mock_dependencies(30, &[])
    }
    
    fn mkenv(sender: impl Into<HumanAddr>) -> Env {
        mock_env(sender, &[])
    }

    impl Into<InitMsg> for &State {
        fn into(self) -> InitMsg {
            InitMsg {
            }
        }
    }
    
}
