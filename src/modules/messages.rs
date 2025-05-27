use cmd_arg::cmd_arg;
use ipak::dprintln;
use colored::Colorize;
use std::env::consts::ARCH;
pub fn help(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    for arg in args {
        dprintln!("Arg: {}", arg);
    }
    Ok(())
}
pub fn manual() -> Result<(), std::io::Error> {
    Ok(())
}
pub fn version() -> Result<(), std::io::Error> {
    println!("{} {} ({})",env!("CARGO_PKG_NAME").bold(),env!("CARGO_PKG_VERSION"),ARCH);
    Ok(())
}
