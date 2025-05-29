use super::messages;
use cmd_arg::cmd_arg;
mod server;
use std::io;
pub fn repo(args: Vec<&cmd_arg::Option>) -> Result<(), io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "serve" | "server" => server::server(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}
