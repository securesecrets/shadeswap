use network_integration::utils::{LPTOKEN20_FILE, print_stored_contract};
use serde::{Deserialize, Serialize};
use secretcli::secretcli::{account_address, init, store_and_return_contract};

pub const ACCOUNT_KEY: &str  = "deployer";
pub const STORE_GAS: &str  = "10000000";

fn main() -> serde_json::Result<()> {
    let s_lp = store_and_return_contract(LPTOKEN20_FILE, ACCOUNT_KEY, Some(STORE_GAS), Some("test"))?;
    print_stored_contract(&s_lp);
    Ok(())
}