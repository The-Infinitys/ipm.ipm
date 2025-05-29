use cmd_arg::cmd_arg;
use tokio::io;
mod install;
mod list;
mod purge;
mod remove;
mod server;
mod update;
use super::messages;
pub fn repo(args: Vec<&cmd_arg::Option>) -> Result<(), io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "update" | "-U" => update::update()?,
        "install" | "-i" => install::install(sub_args)?,
        "remove" => remove::remove(sub_args)?,
        "purge" => purge::purge(sub_args)?,
        "serve" | "server" => server::server(sub_args)?,
        "list" | "--list" | "-l" => list::list(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}
