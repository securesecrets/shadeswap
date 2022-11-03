use std::{io, string};
use std::env;
use std::io::{Write};
use std::io::BufRead;
use network_integration::cli_menu::parse_args;

fn main() 
-> io::Result<()> {     
    let mut reports = vec![];   
    let args: Vec<String> = env::args().collect();
    parse_args(&args, &mut reports)?;
    Ok(())
}