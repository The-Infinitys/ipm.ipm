use cmd_arg::cmd_arg;
use ipak::dprintln;
pub fn install(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    dprintln!("{:?}", args);
    Ok(())
}
