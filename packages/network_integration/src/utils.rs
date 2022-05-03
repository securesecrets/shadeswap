use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use secretcli::{cli_types::NetContract, secretcli::query};
use serde::Serialize;
use std::fmt::Display;
use std::fs;

// Smart contracts
pub const SNIP20_FILE: &str = "../../compiled/snip20_reference_impl.wasm.gz";

pub const STORE_GAS: &str = "10000000";
pub const GAS: &str = "800000";
pub const VIEW_KEY: &str = "password";
pub const ACCOUNT_KEY: &str = "a";

pub fn generate_label(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

pub fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

pub fn print_contract(contract: &NetContract) {
    println!(
        "\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}",
        contract.label, contract.id, contract.address, contract.code_hash
    );
}

pub fn print_struct<Printable: Serialize>(item: Printable) {
    println!("{}", serde_json::to_string_pretty(&item).unwrap());
}

pub fn print_vec<Type: Display>(prefix: &str, vec: Vec<Type>) {
    for e in vec.iter().take(1) {
        print!("{}{}", prefix, e);
    }
    for e in vec.iter().skip(1) {
        print!(", {}", e);
    }
    println!();
}

pub fn store_struct<T: serde::Serialize>(path: &str, data: &T) {
    fs::write(
        path,
        serde_json::to_string_pretty(data).expect("Could not serialize data"),
    )
    .expect(&format!("Could not store {}", path));
}
