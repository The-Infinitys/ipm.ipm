use cmd_arg::cmd_arg;
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
        "fetch" => {
            let pacakges = sub_args
                .iter()
                .map(|arg| -> String { arg.opt_str.to_string() })
                .collect();
            fetch_pkgs(pacakges)
        }
        _ => Err(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )),
    }
}
fn fetch_pkgs(
    packages: Vec<String>,
) -> Result<(), std::io::Error> {
    Ok(())
}
