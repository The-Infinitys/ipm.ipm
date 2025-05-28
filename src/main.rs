use cmd_arg::cmd_arg;
use ipak::dprintln;
use ipak::modules::messages as ipak_messages;
use ipak::modules::pkg as ipak_pkg;
use ipak::modules::project as ipak_project;
use ipak::modules::system as ipak_system;
use ipm::modules::messages as ipm_messages;
use ipm::modules::repo as ipm_repo;
fn main() -> Result<(), std::io::Error> {
    let command_data = cmd_arg::get();
    dprintln!("{}", command_data);
    let opts = command_data.opts;

    // 引数がない場合は早期リターン
    if opts.is_empty() {
        return Err(std::io::Error::from(
            std::io::ErrorKind::InvalidInput,
        ));
    }

    let command = &opts[0];
    let sub_opts: Vec<&cmd_arg::Option> = opts[1..].iter().collect();

    // SubCommand enumの定義はそのまま
    enum SubCommand {
        Help,
        Manual,
        Version,
        Project,
        Package,
        Repository,
        Unknown,
        System,
    }

    let opt_str = command.opt_str.as_str();

    // OptionTypeに関わらず、opt_strで直接マッチング
    let sub_command: SubCommand = match opt_str {
        "--help" | "-h" | "help" => SubCommand::Help,
        "--manual" | "-m" | "manual" | "man" => SubCommand::Manual,
        "--version" | "-v" | "version" => SubCommand::Version,
        "project" | "proj" | "--projec" => SubCommand::Project,
        "system" | "sys" | "--system" => SubCommand::System,
        "pkg" | "package" | "--package" => SubCommand::Package,
        "repo" | "repository" | "repositories" => SubCommand::Repository,
        _ => SubCommand::Unknown,
    };

    match sub_command {
        SubCommand::Help => ipm_messages::help(sub_opts)?,
        SubCommand::Version => ipm_messages::version()?,
        SubCommand::Manual => ipm_messages::manual()?,
        SubCommand::Project => ipak_project::project(sub_opts)?,
        SubCommand::System => ipak_system::system(sub_opts)?,
        SubCommand::Repository => ipm_repo::repo(sub_opts)?,
        SubCommand::Package => ipak_pkg::pkg(sub_opts)?,
        SubCommand::Unknown => ipak_messages::unknown()?,
    }
    Ok(())
}
