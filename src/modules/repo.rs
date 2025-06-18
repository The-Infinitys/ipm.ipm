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
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct RepoSource {
    
    apt: Option<types::apt::Sources>,
    ipm: Option<Vec<URL>>,
}
impl fmt::Display for RepoSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref apt) = self.apt {
            write!(f, ", Apt Source\n {:>2}", apt)?;
        }
        if let Some(ref ipm) = self.ipm {
            write!(f, ", IPM URLs: {}", ipm)?;
        }

        Ok(())
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
    pub fn ipm(data: Vec<URL>) -> Result<Self, std::io::Error> {
        let mut stacked_error = Vec::with_capacity(data.len());
        for url in data {
            let result = types::ipm::fetch(url);
            match result {
                Ok(repo_data) => return Ok(repo_data),
                Err(e) => {
                    let stringify_error=e.to_string();
                    stacked_error.push(stringify_error);
                },
            }
        }
        Err(std::io::Error::other(stacked_error.join(", ")))
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
