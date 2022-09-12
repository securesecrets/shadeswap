pub mod contract;
pub mod operations;
pub mod state;
pub mod help_math;
#[cfg(test)] mod test;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::contract;  
    use cosmwasm_std::ExternalStorage;
    use cosmwasm_std::ExternalApi;
    use cosmwasm_std::ExternalQuerier;
    use cosmwasm_std::do_query;
    use cosmwasm_std::do_init;
    use cosmwasm_std::do_handle;

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
