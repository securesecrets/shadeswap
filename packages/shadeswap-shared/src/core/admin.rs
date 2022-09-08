use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Querier,
    StdError,
    StdResult,
    Storage, Env, Response,
    Deps, DepsMut, MessageInfo, Addr 
};
use cosmwasm_storage::{singleton, Singleton, singleton_read, ReadonlySingleton};

pub static ADMIN: &[u8] =b"contract_pair_admin";

pub fn admin_w(storage: &mut dyn Storage) -> Singleton<Addr> {
    singleton(storage, ADMIN)
}

pub fn admin_r(storage: & dyn Storage) -> ReadonlySingleton<Addr> {
    singleton_read(storage, ADMIN)
}

pub fn apply_admin_guard(
    caller: &Addr,
    storage: &mut dyn Storage,
) -> StdResult<bool> {    
    let admin_address = admin_r(storage).load()?;
    if caller.as_str() != admin_address.as_str() {
         return Err(StdError::generic_err("Caller is not admin"))
    }
    return Ok(true)
}

// pub fn set_admin_guard<S: Storage, A: Api, Q: Querier>(
//     storage: &mut dyn Storage,
//     env: Env,
//     info: MessageInfo,
//     admin: String
// ) -> StdResult<Response>{
//     let sender = info.sender.to_string();
//     apply_admin_guard(sender.clone(), storage)?;
//     store_admin(storage,&admin);
//     Ok(Response::new()
//     .add_attribute("action", "set_admin_guard")
//     .add_attribute("caller", sender.clone())
//     .add_attribute("admin", admin))
// }