use std::fmt;
use tokio::io;
use ipak::modules::pkg::PackageData;

mod apt;
mod ipm;

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
    pub fn new(repo_type: RepoType, apt_info: Option<apt::AptInfo>, ipm_info: Option<ipm::IpmInfo>) -> Self {
        Self {
            repo_type,
            apt_info,
            ipm_info,
        }
    }

    /// パッケージデータを取得する非同期関数
    pub async fn get(&self) -> Result<Vec<PackageData>, Box<dyn std::error::Error>> {
        match self.repo_type {
            RepoType::Apt => {
                if let Some(apt_info) = &self.apt_info {
                    let packages = apt_info.get_packages().await?;
                    Ok(packages.into_iter().map(PackageData::from).collect())
                } else {
                    Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "AptInfo is missing")))
                }
            }
            RepoType::Ipm => {
                if let Some(ipm_info) = &self.ipm_info {
                    let package_list_data = ipm_info.get_packages().await?;
                    Ok(package_list_data.installed_packages.into_iter().map(|data|->PackageData{data.info}).collect())
                } else {
                    Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "IpmInfo is missing")))
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