use super::super::messages;
use chrono::{DateTime, Local};
use cmd_arg::cmd_arg;
use ipak::modules::pkg::PackageData;
mod init;
use std::fmt;

pub fn server(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "init" => init::init(sub_args)?,
        _ => messages::unknown()?,
    }
    Ok(())
}

pub struct RepoIndex {
    url: String,
    last_updated: Option<DateTime<Local>>,
    packages: Vec<PackageData>,
}

impl RepoIndex {
    /// `RepoIndex` の新しいインスタンスを作成するためのコンストラクタ
    pub fn new(url: String) -> Self {
        Self { url, last_updated: None, packages: Vec::new() }
    }

    /// パッケージデータを取得する非同期関数
    pub async fn get(
        &mut self,
    ) -> Result<&Vec<PackageData>, Box<dyn std::error::Error>> {
        if self.packages.is_empty() {
            // パッケージリストの取得
            self.update().await?;
        }
        Ok(&self.packages)
    }

    /// リポジトリの情報を更新する
    async fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // リポジトリからパッケージ情報を取得する処理
        // TODO: 実際のリポジトリからデータを取得する実装
        self.last_updated = Some(Local::now());
        Ok(())
    }
}

impl fmt::Display for RepoIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Repository: {}\nLast updated: {}",
            self.url,
            self.last_updated
                .map_or("Never".to_string(), |dt| dt.to_string())
        )
    }
}

#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub url: String,
    pub last_modified: DateTime<Local>,
    pub info: PackageData,
}
