use cmd_arg::cmd_arg;
use ipak::modules::pkg as ipak_pkg;
pub fn pkg(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    ipak_pkg::pkg(args)
}
