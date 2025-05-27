use cmd_arg::cmd_arg;
use ipak::dprintln;
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
    Ok(())
}
