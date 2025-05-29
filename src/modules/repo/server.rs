use super::super::messages;
use cmd_arg::cmd_arg;
mod init;
mod metadata;
mod project;
pub fn server(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "init" | "-i" => init::init(sub_args)?,
        "project" | "proj" => project::project(sub_args)?,
        "metadata" | "info" => metadata::show_metadata()?,
        _ => messages::unknown()?,
    }
    Ok(())
}
