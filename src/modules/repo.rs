use cmd_arg::cmd_arg;
use ipak::modules::pkg::PackageData;
use std::fmt;
use tokio::io;
mod install;
mod list;
mod purge;
mod remove;
mod server;
mod types;
mod update;
use super::messages;
use types::ipm;
pub struct RepoIndex {
    ipm_info: ipm::IpmInfo,
}

impl RepoIndex {
    /// `RepoIndex` の新しいインスタンスを作成するためのコンストラクタ
    pub fn new(ipm_info: ipm::IpmInfo) -> Self {
        Self { ipm_info }
    }

    /// パッケージデータを取得する非同期関数
    pub async fn get(
        &self,
    ) -> Result<Vec<PackageData>, Box<dyn std::error::Error>> {
        let ipm_info = &self.ipm_info;
        let package_list_data = ipm_info.get_packages().await?;
        Ok(package_list_data
            .installed_packages
            .into_iter()
            .map(|data| -> PackageData { data.info })
            .collect())
    }
}

impl fmt::Display for RepoIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ipm_info)
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
        "list" | "--list" | "-l" => list::list(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}
