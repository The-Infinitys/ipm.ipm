use super::messages;
use crate::utils::www::*;
use chrono::{DateTime, Local};
use cmd_arg::cmd_arg;
use ipak::dprintln;
use ipak::modules::pkg::{AuthorAboutData, PackageData};
use ipak::utils::color::colorize::*;
mod server;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_yaml;
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
    author: AuthorAboutData,
    last_modified: DateTime<Local>,
    packages: Vec<PackageMetaData>,
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
    last_modified: DateTime<Local>,
    info: PackageData,
    url: String,
}
impl RepoData {
    pub fn new(
        repo_type: RepoType,
        url: URL,
    ) -> Result<Self, std::io::Error> {
        dprintln!("{}", repo_type);
        let url = format!("{}/repo.yaml", url);
        let request = reqwest::blocking::get(url).map_err(
            |e| -> std::io::Error { std::io::Error::other(e) },
        )?;
        let request =
            request.text().map_err(|e| -> std::io::Error {
                std::io::Error::other(e)
            })?;
        let result: RepoData = serde_yaml::from_str(&request)
            .map_err(|e| -> std::io::Error {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )
            })?;
        Ok(result)
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
    fn test() -> Result<(), std::io::Error> {
        let test_repodata = RepoData::new(
            RepoType::Ipm,
            "http://develop.the-infinitys.f5.si/ipm.official-repo/".to_url().unwrap(),
        )?;
        println!("{}", test_repodata);
        Ok(())
    }
}
