use super::messages;
use crate::utils::www::*;
use chrono::{DateTime, Local};
use cmd_arg::cmd_arg;
use ipak::modules::pkg::{AuthorAboutData, PackageData};
use ipak::utils::color::colorize::*;
mod list;
mod pkg;
mod server;
pub mod types;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{fmt, io};

pub fn repo(
    args: Vec<&cmd_arg::Option>,
) -> Result<(), io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "serve" | "server" => server::server(sub_args)?,
        "pkg" | "package" => pkg::pkg(sub_args)?,
        "list" => list::list()?,
        _ => messages::unknown()?,
    }
    Ok(())
}
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub enum RepoType {
    #[default]
    Ipm,
    Apt,
}
impl FromStr for RepoType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "apt" => Ok(Self::Apt),
            "ipm" => Ok(Self::Ipm),
            _ => Err(format!("Invalid RepoType: {}", s)),
        }
    }
}
impl fmt::Display for RepoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipm => write!(f, "ipm"),
            Self::Apt => write!(f, "apt"),
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct RepoData {
    pub author: AuthorAboutData, // pub に変更してテストでアクセス可能に
    pub last_modified: DateTime<Local>, // pub に変更してテストでアクセス可能に
    pub packages: Vec<PackageMetaData>, // pub に変更してテストでアクセス可能に
}
impl Default for RepoData {
    fn default() -> Self {
        Self {
            author: AuthorAboutData::default(),
            last_modified: Local::now(),
            packages: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackageMetaData {
    pub last_modified: DateTime<Local>, // pub に変更してテストでアクセス可能に
    pub info: PackageData, // pub に変更してテストでアクセス可能に
    pub url: String, // pub に変更してテストでアクセス可能に
}
impl RepoData {
    pub fn new(
        repo_type: RepoType,
        url: URL,
    ) -> Result<Self, std::io::Error> {
        match repo_type {
            RepoType::Ipm => types::ipm::fetch(url),
            RepoType::Apt => types::apt::fetch(url),
        }
    }
}
impl fmt::Display for PackageMetaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.info)?;
        writeln!(
            f,
            "{}: {}",
            "Last Modified".bold(),
            self.last_modified
        )?;
        writeln!(f, "{}: {}", "URL".bold(), self.url)
    }
}

impl fmt::Display for RepoData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:\n{}", "Author".bold(), self.author)?;
        writeln!(
            f,
            "{}: {}",
            "Last Modified".bold(),
            self.last_modified
        )?;
        for package in &self.packages {
            writeln!(f, "{}", package)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_ipm_repo() -> Result<(), std::io::Error> {
        println!("Testing IPM Repository Fetch...");
        let test_url = "https://develop.the-infinitys.f5.si/ipm.official-repo/".to_url().unwrap();
        let test_repodata =
            RepoData::new(RepoType::Ipm, test_url.clone())?;
        println!(
            "Successfully fetched IPM repo from: {}",
            test_url
        );
        println!("{}", test_repodata);
        // 基本的なアサーション
        assert!(
            !test_repodata.packages.is_empty(),
            "IPM repo should contain packages."
        );
        Ok(())
    }

    #[test]
    // #[ignore = "External network dependency, might be slow or unstable"]
    fn test_fetch_apt_repo() -> Result<(), std::io::Error> {
        println!("\nTesting APT Repository Fetch...");
        // Debianの安定版リポジトリのURLを使用
        let test_url = "https://archive.ubuntu.com/ubuntu/dists/plucky/main/binary-amd64/".to_url().unwrap();
        let test_repodata =
            RepoData::new(RepoType::Apt, test_url.clone())?;
        println!(
            "Successfully fetched APT repo from: {}",
            test_url
        );
        println!("{}", test_repodata);

        // 基本的なアサーション
        assert!(
            !test_repodata.packages.is_empty(),
            "APT repo should contain packages."
        );
        assert!(
            test_repodata.author.name
                != AuthorAboutData::default().name,
            "APT repo author should be set."
        );
        assert!(
            test_repodata.last_modified != Local::now()
                && (Local::now() - test_repodata.last_modified)
                    .num_days()
                    < 365,
            "APT repo last modified date should be recent."
        );

        // assert!(apt_package_found, "The 'apt' package should be found in the Debian repository.");

        Ok(())
    }
}
