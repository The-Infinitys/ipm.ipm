use cmd_arg::cmd_arg;
use ipak::dprintln;
use ipak::modules::{messages, pkg, project, system};
fn main() -> Result<(), std::io::Error> {
    let command_data = cmd_arg::get();
    dprintln!("{}", command_data);
    let opts = command_data.opts;

    // 引数がない場合は早期リターン
    if opts.is_empty() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
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
        _ => SubCommand::Unknown,
    };

    match sub_command {
        SubCommand::Help => messages::help(sub_opts)?,
        SubCommand::Version => messages::version()?,
        SubCommand::Manual => messages::manual()?,
        SubCommand::Project => project::project(sub_opts)?,
        SubCommand::System => system::system(sub_opts)?,
        SubCommand::Package => pkg::pkg(sub_opts)?,
        SubCommand::Unknown => messages::unknown()?,
    }

    println!("Hello, world!");
    Ok(())
}
