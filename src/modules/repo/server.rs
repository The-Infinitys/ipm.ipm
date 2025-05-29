use super::super::messages;
use chrono::{DateTime, Local};
use cmd_arg::cmd_arg;
use ipak::modules::pkg::{AuthorAboutData, PackageData};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
mod init;

pub fn server(args: Vec<&cmd_arg::Option>) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }
    let sub_cmd = args.first().unwrap().to_owned();
    // let sub_args: Vec<&cmd_arg::Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "init" => init::init()?,
        _ => messages::unknown()?,
    }
    Ok(())
}

pub struct RepoIndex {
    url: String,
    pub author: AuthorAboutData,
    last_updated: Option<DateTime<Local>>,
    packages: Vec<PackageMetadata>,
}

impl RepoIndex {
    /// `RepoIndex` の新しいインスタンスを作成するためのコンストラクタ
    pub fn new(url: String) -> Self {
        Self {
            url,
            author: AuthorAboutData::default(),
            last_updated: None,
            packages: Vec::new(),
        }
    }

    /// パッケージデータを取得する関数
    pub fn get(
        &mut self,
    ) -> Result<Vec<PackageData>, Box<dyn std::error::Error>> {
        if self.packages.is_empty() {
            // パッケージリストの取得
            self.update()?;
        }
        Ok(self.packages.iter().map(|pkg| pkg.info.to_owned()).collect())
    }

    /// リポジトリの情報を更新する
    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_url = format!("{}/repo.yaml", self.url);

        // HTTP GETリクエストを送信
        let client = Client::new();
        let response = client.get(&yaml_url).send()?;

        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "リポジトリの取得に失敗しました: HTTP {}",
                    response.status()
                ),
            )));
        }

        // YAMLレスポンスをパース
        let repo_yaml: RepoYaml = serde_yaml::from_str(&response.text()?)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("YAMLのパースに失敗しました: {}", e),
                )
            })?;

        // 取得したデータでリポジトリ情報を更新
        self.url = repo_yaml.url;
        self.author = repo_yaml.author;
        self.packages = repo_yaml.packages;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub url: String,
    pub last_modified: DateTime<Local>,
    pub info: PackageData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoYaml {
    url: String,
    author: AuthorAboutData,
    last_updated: Option<DateTime<Local>>,
    packages: Vec<PackageMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_update() {
        // テスト用のYAMLファイルを作成
        let test_dir = tempfile::tempdir().unwrap();
        let yaml_path = test_dir.path().join("repo.yaml");
        let yaml_content = r#"
            url: "http://test-server"
            author:
              name: "Test Author"
              email: "test@example.com"
            last_updated: "2025-05-29T10:00:00+09:00"
            packages: []
        "#;
        fs::write(&yaml_path, yaml_content).unwrap();

        // ローカルファイルシステムを使用してテスト
        let mut repo = RepoIndex::new(format!("file://{}", yaml_path.display()));
        let result = repo.update();
        
        assert!(result.is_ok());
        assert_eq!(repo.author.name, "Test Author");
        assert_eq!(repo.author.email, "test@example.com");
    }
}
