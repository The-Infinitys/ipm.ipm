use cmd_arg::cmd_arg;
use ipak::modules::pkg::PackageData;
use std::fmt;
use tokio::io;
mod install;
mod purge;
mod remove;
mod server;
mod types;
mod update;
use super::messages;
use types::{apt, ipm};

pub struct RepoIndex {
    repo_type: RepoType,
    apt_info: Option<apt::AptInfo>,
    ipm_info: Option<ipm::IpmInfo>,
}

pub enum RepoType {
    Apt,
    Ipm,
}

impl RepoIndex {
    /// `RepoIndex` の新しいインスタンスを作成するためのコンストラクタ
    pub fn new_apt(apt_info: apt::AptInfo) -> Self {
        Self {
            repo_type: RepoType::Apt,
            apt_info: Some(apt_info),
            ipm_info: None,
        }
    }
    pub fn new_ipm(ipm_info: ipm::IpmInfo) -> Self {
        Self {
            repo_type: RepoType::Ipm,
            apt_info: None,
            ipm_info: Some(ipm_info),
        }
    }
    /// パッケージデータを取得する非同期関数
    pub async fn get(
        &self,
    ) -> Result<Vec<PackageData>, Box<dyn std::error::Error>> {
        match self.repo_type {
            RepoType::Apt => {
                if let Some(apt_info) = &self.apt_info {
                    let packages = apt_info.get_packages().await?;
                    Ok(packages
                        .into_iter()
                        .map(PackageData::from)
                        .collect())
                } else {
                    Err(Box::new(io::Error::new(
                        io::ErrorKind::NotFound,
                        "AptInfo is missing",
                    )))
                }
            }
            RepoType::Ipm => {
                if let Some(ipm_info) = &self.ipm_info {
                    let package_list_data =
                        ipm_info.get_packages().await?;
                    Ok(package_list_data
                        .installed_packages
                        .into_iter()
                        .map(|data| -> PackageData { data.info })
                        .collect())
                } else {
                    Err(Box::new(io::Error::new(
                        io::ErrorKind::NotFound,
                        "IpmInfo is missing",
                    )))
                }
            }
        }
    }
}

impl fmt::Display for RepoIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repo_type {
            RepoType::Apt => {
                if let Some(apt_info) = &self.apt_info {
                    write!(f, "RepoType: Apt\n{}", apt_info)
                } else {
                    write!(f, "RepoType: Apt\nAptInfo is missing")
                }
            }
            RepoType::Ipm => {
                if let Some(ipm_info) = &self.ipm_info {
                    write!(f, "RepoType: Ipm\n{}", ipm_info)
                } else {
                    write!(f, "RepoType: Ipm\nIpmInfo is missing")
                }
            }
        }
    }
}

pub fn repo(args: Vec<&cmd_arg::Option>) -> Result<(), io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "update" | "-U" => update::update()?,
        "install" | "-i" => install::install(sub_args)?,
        "remove" => remove::remove(sub_args)?,
        "purge" => purge::purge(sub_args)?,
        "serve" | "server" => server::server(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_repo_index_new() {
        let apt_info = apt::AptInfo {
            types: apt::AptType::Deb,
            uris: "http://example.com".to_string(),
            suites: vec!["stable".to_string()],
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        };

        let repo_index = RepoIndex::new_apt(apt_info);
        assert!(repo_index.ipm_info.is_none());
    }

    #[tokio::test]
    async fn test_repo_index_get_apt() {
        let apt_info = apt::AptInfo {
            types: apt::AptType::Deb,
            uris: "http://archive.ubuntu.com/ubuntu".to_string(),
            suites: vec!["noble".to_string()],
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        };

        let repo_index = RepoIndex::new_apt(apt_info);

        let result = repo_index.get().await;
        match result {
            Ok(packages) => {
                assert!(
                    !packages.is_empty(),
                    "Packages should not be empty"
                );
            }
            Err(e) => panic!("Expected valid response, got error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_repo_index_display() {
        let apt_info = apt::AptInfo {
            types: apt::AptType::Deb,
            uris: "http://example.com".to_string(),
            suites: vec!["stable".to_string()],
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        };

        let repo_index_apt = RepoIndex::new_apt(apt_info);

        assert!(format!("{}", repo_index_apt).contains("RepoType: Apt"));
    }
}
