use cmd_arg::cmd_arg;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Deserialize)]
struct RepoConfig {
    url: String,
    last_updated: Option<DateTime<Local>>,
    projects: Vec<PathBuf>,
}

impl RepoConfig {
    fn new() -> Self {
        Self {
            url: "http://localhost:8000".to_string(),
            last_updated: Some(Local::now()),
            projects: vec![PathBuf::from("projects/ipak")],
        }
    }

    fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        // YAMLにシリアライズ
        let yaml = serde_yaml::to_string(&self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // ファイルに書き込み
        let mut file = File::create(path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }

    fn load(path: &PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

pub fn init() -> Result<(), std::io::Error> {
    let repo_path = PathBuf::from("ipm/repo.yaml");

    // ディレクトリが存在しない場合は作成
    if let Some(parent) = repo_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // リポジトリ設定の作成と保存
    let config = RepoConfig::new();
    config.save(&repo_path)?;

    println!("Repository initialized successfully!");
    Ok(())
}
