use cosmwasm_std::{StdResult, Addr};

pub fn try_addr_validate_option(api: &dyn cosmwasm_std::Api, addr: Option<String>) -> StdResult<Option<Addr>>
{
    return match addr {
        Some(a) => Ok(Some(api.addr_validate(&a)?)),
        None => Ok(None),
    };
}