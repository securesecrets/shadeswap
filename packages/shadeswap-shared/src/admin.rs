use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Extern,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage, Env, HandleResponse, log,
};

use crate::scrt_storage::{load, save, ns_save, ns_load};

pub static ADMIN: &[u8] =b"contract_pair_admin";

pub fn apply_admin_guard(
    caller: HumanAddr,
    storage: &impl Storage,
) -> StdResult<bool> {    
    let admin_address = load_admin(storage)?;
    if caller.as_str() != admin_address.as_str() {
         return Err(StdError::unauthorized())
    }
    return Ok(true)
}

pub fn store_admin <S: Storage, A: Api, Q: Querier>(
    deps:  &mut Extern<S, A, Q>,
    admin: &HumanAddr
) -> StdResult<()> {
    save(&mut deps.storage, ADMIN, &admin)
}

pub fn load_admin(storage: &impl Storage) -> StdResult<HumanAddr> {
    let admin = load(storage, ADMIN)?.unwrap_or(HumanAddr("".to_string()));
    Ok(admin)
}

pub fn set_admin_guard<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>, 
    env: Env,
    admin: HumanAddr
) -> StdResult<HandleResponse>{
    let sender = env.message.sender.clone();
    apply_admin_guard(sender.clone(), &deps.storage)?;
    store_admin(deps,&admin)?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![
                log("action", "set_admin_guard"),
                log("caller", sender.clone()),
                log("admin", admin),
        ],
        data: None,
    })

}