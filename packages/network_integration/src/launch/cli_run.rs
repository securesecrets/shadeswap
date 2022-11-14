use std::{io};
use std::env;


use network_integration::cli_menu::parse_args;

fn main() 
-> io::Result<()> {     
    let mut reports = vec![];   
    let args: Vec<String> = env::args().collect();
    parse_args(&args, &mut reports)?;
    Ok(())
}