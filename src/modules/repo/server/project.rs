use super::super::super::messages;
use cmd_arg::cmd_arg;
pub fn project(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    // let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        // "add" => project_add(sub_args)?,
        // "remove" => project_remove(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}
// fn project_add(
//     args: Vec<&cmd_arg::Option>,
// ) -> Result<(), std::io::Error> {
//     Ok(())
// }
// fn project_remove(
//     args: Vec<&cmd_arg::Option>,
// ) -> Result<(), std::io::Error> {
//     Ok(())
// }
