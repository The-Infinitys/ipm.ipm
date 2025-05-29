use super::super::messages;
use cmd_arg::cmd_arg;
use ipak::dprintln;
pub fn server(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    // let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    dprintln!("{}", sub_cmd);
    Ok(())
}
