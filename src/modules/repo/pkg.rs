use std::env;

use super::list;
use crate::modules::repo::PackageMetaData;
use crate::utils::www::*;
use cmd_arg::cmd_arg;
use ipak::utils::color::colorize::Colorize;
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
        "search" => {
            let pacakges = sub_args
                .iter()
                .map(|arg| -> String { arg.opt_str.to_string() })
                .collect();
            show_searched_pkgs(pacakges)
        }
        "fetch" => {
            let packages = sub_args
                .iter()
                .map(|arg| -> String { arg.opt_str.to_string() })
                .collect();
            fetch_pkg(packages)
        }
        _ => Err(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )),
    }
}
fn show_searched_pkgs(
    packages_name: Vec<String>,
) -> Result<(), std::io::Error> {
    let packages = search_pkgs(packages_name.clone())?;
    if packages.is_empty() {
        eprintln!(
            "{}: {}",
            "パッケージが見つかりませんでした。".red(),
            packages_name.join(", ")
        )
    }
    for package in packages {
        println!("{}", package);
    }
    Ok(())
}
fn search_pkgs(
    packages_name: Vec<String>,
) -> Result<Vec<PackageMetaData>, std::io::Error> {
    let packages = list::packages()?;
    let packages = packages
        .into_iter()
        .filter(|p| {
            for name in &packages_name {
                if &p.info.about.package.name == name {
                    return true;
                }
            }
            false
        })
        .collect();
    Ok(packages)
}
fn fetch_pkg(
    packages_name: Vec<String>,
) -> Result<(), std::io::Error> {
    let packages = search_pkgs(packages_name.clone())?;
    if packages.is_empty() {
        eprintln!(
            "{}: {}",
            "パッケージが見つかりませんでした。".red(),
            packages_name.join(", ")
        )
    }
    for package in packages {
        let target_url = package
            .url
            .to_url()
            .map_err(std::io::Error::other)?;
        let filename = match target_url.path().file_name() {
            Some(path_os_str) => {
                path_os_str.to_string_lossy().to_string()
            }
            None => {
                format!(
                    "{}-{}.package",
                    package.info.about.package.name,
                    package.info.about.package.version
                )
            }
        };
        let target_path = env::current_dir()?.join(filename);
        let data = target_url
            .fetch_bin()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(target_path, &data)?;
    }
    Ok(())
}
