use std::io::{self, Write, Error, ErrorKind};

use secretcli::secretcli::Report;

use crate::cli_commands::snip20_lib::create_new_snip_20;

pub const HELP: &str = "help";
pub const CREATESNIP20: &str = "snip20";


pub fn parse_args(args: &[String], reports: &mut Vec<Report>) -> io::Result<()>
{
    if args.len() == 0 {
        return Err(Error::new(ErrorKind::Other, "not enough arguments"));
    }  

    let args_command = args[1].clone();
    println!("{}", args_command);
    if args_command == HELP{
        print_help()?;
    }
   
    if args_command == CREATESNIP20 {
        if args.len() != 8 {
            return Err(Error::new(ErrorKind::Other, "Please provide all args"));
        } 

        let account_name = args[2].clone();
        let backend = args[3].clone();
        let name = args[4].clone();
        let symbol = args[5].clone();
        let decimal = args[6].clone();
        let viewing_key = args[7].clone();
        create_new_snip_20(&account_name, &backend, &name, &symbol,decimal.parse<u64>(), &viewing_key, reports)?;
    }

    Ok(())
}

pub fn print_help() -> io::Result<()>
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(b"Welcome to the Shadeswap CLI.")?;
    handle.write_all(b"\n\t1. Command:: snip20 <account_name> <keyring_backend> <name> <symbol> <decimal> <viewing_key> -- Create new Snip20 Contract")?;
    handle.write_all(b"\n\t")?;
    // handle.write_all(b"\n\t3. Create New AMMPair Contract. ")?;
    // handle.write_all(b"\n\t4. Create Snip20 And Create New AMM Pair. ")?;
    // handle.write_all(b"\n\t5. Mint20 Snip20. \n\t")?;
    // handle.write_all(b"\n\t10. Exit Secretd Cli. \n\t")?;
    handle.flush()?;
  
    Ok(())
}

// fn print_options() -> io::Result<()>
// {
   
// }

