use super::super::super::messages;
use cmd_arg::cmd_arg;
use ipak::modules::project;
use std::{env, fs};
pub fn project(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "add" => project_add(sub_args)?,
        "remove" => project_remove(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}
fn project_add(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    let original_dir = env::current_dir()?;
    env::set_current_dir("projects")?;
    let mut args = args;
    let new_opt = cmd_arg::Option {
        opt_str: "new".to_owned(),
        opt_type: cmd_arg::OptionType::Simple,
        opt_values: vec![],
    };
    args.insert(0, &new_opt);
    let args = args;
    let creation_result = project::project(args);
    env::set_current_dir(original_dir)?;
    creation_result
}
fn project_remove(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    for arg in args {
        if arg.opt_type != cmd_arg::OptionType::Simple {
            continue;
        }
        let target_path =
            format!("projects/{}", arg.opt_str.to_owned());
        fs::remove_dir_all(target_path)?;
    }
    Ok(())
}
