use crate::query_auth::QueryPermit;
use crate::utils::Query;
use crate::{query_auth, Contract};
use cosmwasm_std::{from_binary, Addr, Deps, QuerierWrapper, StdError, StdResult};
use serde::de::DeserializeOwned;

pub struct PermitAuthentication<T: DeserializeOwned> {
    pub sender: Addr,
    pub revoked: bool,
    pub data: T,
}

pub fn authenticate_permit<T: DeserializeOwned>(
    deps: Deps,
    permit: QueryPermit,
    querier: &QuerierWrapper,
    authenticator: Option<Contract>,
) -> StdResult<PermitAuthentication<T>> {
    let sender: Addr;
    let revoked: bool;
    match authenticator {
        Some(a) => {
            let res: query_auth::QueryAnswer = query_auth::QueryMsg::ValidatePermit {
                permit: permit.clone(),
            }
            .query(querier, &a)?;

            match res {
                query_auth::QueryAnswer::ValidatePermit { user, is_revoked } => {
                    sender = user;
                    revoked = is_revoked;
                }
                _ => return Err(StdError::generic_err("Wrong query response")),
            }

            return Ok(PermitAuthentication {
                sender,
                revoked,
                data: from_binary(&permit.params.data)?,
            });
        }
        None => {
            sender = permit.validate(deps.api, None)?.as_addr(None)?;
            return Ok(PermitAuthentication {
                sender,
                revoked: false,
                data: from_binary(&permit.params.data)?,
            });
        }
    }
}

pub fn authenticate_vk(
    address: Addr,
    key: String,
    querier: &QuerierWrapper,
    authenticator: &Contract,
) -> StdResult<bool> {
    let res: query_auth::QueryAnswer =
        query_auth::QueryMsg::ValidateViewingKey { user: address, key }
            .query(querier, authenticator)?;

    match res {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => Ok(is_valid),
        _ => Err(StdError::generic_err("Unauthorized")),
    }
}
