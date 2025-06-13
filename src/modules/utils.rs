use cmd_arg::cmd_arg;
use ipak::modules::utils as ipak_utils;
pub fn utils(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    // 引数がない場合は早期リターン
    if args.is_empty() {
        return Err(std::io::Error::from(
            std::io::ErrorKind::InvalidInput,
        ));
    }
    // let sub_cmd =
        // args.first().unwrap().to_owned().opt_str.clone();
    // let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    // match sub_cmd.as_str() {
        // _ => ipak_utils::utils(args)?,
    // }
    ipak_utils::utils(args)?;
    Ok(())
}
