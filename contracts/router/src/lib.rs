pub mod contract;
pub mod state;
pub mod test;
pub mod operations;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::contract;
    use cosmwasm_std::{
        do_handle,
        do_init,
        do_query,
        ExternalApi,
        ExternalQuerier,
        ExternalStorage,
    };


    #[no_mangle]
    extern "C" fn instantiate(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_init(
            &contract::instantiate::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn execute(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_handle(
            &contract::execute::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn query(msg_ptr: u32) -> u32 {
        do_query(
            &contract::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
            msg_ptr,
        )
    }

    // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
    // automatically because we `use cosmwasm_std`.
}
