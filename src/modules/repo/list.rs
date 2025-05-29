use cmd_arg::cmd_arg;
use ipak::dprintln;
pub fn list(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    dprintln!("{:?}", args);
    Ok(())
}
