use super::super::project;
use super::super::system;
use cmd_arg::cmd_arg;
use ipak::dprintln;
use ipak::modules::project as ipak_project;
use ipak::utils::archive;
use std::path::PathBuf;
use std::{env, fs};
pub fn convert(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), std::io::Error> {
    let mut path_from = String::new();
    let mut path_to = String::new();
    for arg in args {
        match arg.opt_str.as_str() {
            "--from" => {
                if let Some(val) = arg.opt_values.first() {
                    path_from = val.clone();
                }
            }
            "--to" => {
                if let Some(val) = arg.opt_values.first() {
                    path_to = val.clone();
                }
            }
            _ => continue,
        }
    }
    let path_from = PathBuf::from(path_from);
    let path_to = PathBuf::from(path_to);
    dprintln!(
        "Converting package from: {} to: {}",
        path_from.display(),
        path_to.display()
    );
    let cache_dir = system::path::local::cache_dir();
    if !cache_dir.is_dir() {
        fs::create_dir_all(&cache_dir)?;
    }
    let dest_path = cache_dir
        .join(path_from.file_name().unwrap_or_default());
    let extracted_path = cache_dir.join(format!(
        "{}{}",
        path_from
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
        ".d"
    ));

    fs::copy(&path_from, &dest_path)?;
    if !extracted_path.is_dir() {
        fs::create_dir_all(&extracted_path)?;
    }
    archive::extract_archive(
        dest_path.clone(),
        extracted_path.clone(),
    )?;
    let original_current = env::current_dir()?;
    env::set_current_dir(&extracted_path)?;
    let convert_result: Result<(), std::io::Error> = {
        project::convert::convert()?;
        ipak_project::project(vec![&cmd_arg::Option {
            opt_str: "package".to_string(),
            ..Default::default()
        }])?;
        let ipak_dir = extracted_path.join("ipak/package");
        if ipak_dir.exists() && ipak_dir.is_dir() {
            for entry in fs::read_dir(&ipak_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "ipak" {
                        let file_name =
                            path.file_name().unwrap();
                        let target_path =
                            original_current.join(&path_to).join(file_name);
                        fs::rename(&path, &target_path)?;
                    }
                }
            }
        }
        Ok(())
    };
    env::set_current_dir(&original_current)?;
    convert_result?;
    dprintln!(
        "Copied {} to {}",
        path_from.display(),
        dest_path.display()
    );
    Ok(())
}
