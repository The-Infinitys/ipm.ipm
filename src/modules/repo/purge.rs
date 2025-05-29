use cmd_arg::cmd_arg;
use ipak::dprintln;
pub fn purge(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    dprintln!("{:?}",args);
    Ok(())
}
