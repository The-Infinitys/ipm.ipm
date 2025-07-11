use cmd_arg::cmd_arg;
use ipak::modules::pkg as ipak_pkg;
pub mod convert;
pub fn pkg(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return Err(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        ));
    }
    let sub_cmd = args.first().unwrap().opt_str.to_string();
    let sub_args = args[1..].to_vec();
    match sub_cmd.as_str() {
        "convert" => convert::convert(sub_args),
        _ => ipak_pkg::pkg(args),
    }
}
