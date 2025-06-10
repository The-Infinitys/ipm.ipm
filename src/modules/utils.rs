use super::messages;
use cmd_arg::cmd_arg;
use ipak::utils as ipak_utils;
use std::path::PathBuf;
use std::str::FromStr;
pub fn utils(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    // 引数がない場合は早期リターン
    if args.is_empty() {
        return Err(std::io::Error::from(
            std::io::ErrorKind::InvalidInput,
        ));
    }
    let sub_cmd =
        args.first().unwrap().to_owned().opt_str.clone();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.as_str() {
        "archive" => archive(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}
fn archive(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return Err(std::io::Error::from(
            std::io::ErrorKind::InvalidInput,
        ));
    }
    let sub_cmd = args.first().unwrap().opt_str.as_str();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd {
        "create" => create_archive(sub_args)?,
        "extract" => exracte_archive(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}

fn create_archive(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    let mut from_path = "";
    let mut to_path = "";
    let mut archive_type = "";
    for arg in args {
        match arg.opt_str.as_str() {
            "--from" => {
                if let Some(s) = arg.opt_values.get(0) {
                    from_path = s;
                }
            }
            "--to" => {
                if let Some(s) = arg.opt_values.get(0) {
                    to_path = s;
                }
            }
            "--type" => {
                if let Some(s) = arg.opt_values.get(0) {
                    archive_type = s;
                }
            }
            _ => continue,
        }
    }
    let from_path = PathBuf::from(from_path);
    let to_path = PathBuf::from(to_path);
    let archive_type =
        ipak_utils::archive::ArchiveType::from_str(archive_type)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown Archive Type: {}", e),
                )
            })?;
    ipak_utils::archive::create_archive(
        from_path,
        to_path,
        archive_type,
    )?;
    Ok(())
}
fn exracte_archive(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    let mut from_path = "";
    let mut to_path = "";
    for arg in args {
        match arg.opt_str.as_str() {
            "--from" => {
                if let Some(s) = arg.opt_values.get(0) {
                    from_path = s;
                }
            }
            "--to" => {
                if let Some(s) = arg.opt_values.get(0) {
                    to_path = s;
                }
            }
            _ => continue,
        }
    }
    let from_path = PathBuf::from(from_path);
    let to_path = PathBuf::from(to_path);
    ipak_utils::archive::extract_archive(from_path, to_path)?;
    Ok(())
}
