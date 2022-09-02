use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Querier,
    StdError,
    StdResult,
    Storage, Env, Response,
    Deps 
};

use crate::scrt_storage::{load, save, ns_save, ns_load};

pub static ADMIN: &[u8] =b"contract_pair_admin";

pub fn apply_admin_guard(
    caller: String,
    storage: &impl Storage,
) -> StdResult<bool> {    
    let admin_address = load_admin(storage)?;
    if caller.as_str() != admin_address.as_str() {
         return Err(StdError::unauthorized())
    }
    return Ok(true)
}

pub fn store_admin <S: Storage, A: Api, Q: Querier>(
    deps:  &mut Deps<S, A, Q>,
    admin: &String
) -> StdResult<()> {
    save(&mut deps.storage, ADMIN, &admin)
}

pub fn load_admin(storage: &impl Storage) -> StdResult<String> {
    let admin = load(storage, ADMIN)?.unwrap_or("".to_string());
    Ok(admin)
}

pub fn set_admin_guard<S: Storage, A: Api, Q: Querier>(
    deps: &mut Deps<S, A, Q>, 
    env: Env,
    admin: String
) -> StdResult<Response>{
    let sender = env.message.sender.clone();
    apply_admin_guard(sender.clone(), &deps.storage)?;
    store_admin(deps,&admin)?;
    Ok(Response::new()
    .add_attribute("action", "set_admin_guard")
    .add_attribute("caller", sender.clone())
    .add_attribute("admin", admin))
}