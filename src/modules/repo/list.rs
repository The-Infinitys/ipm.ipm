// src/modules/repo/list.rs
use crate::modules::{repo::RepoType, system::path};
use crate::modules::repo::{RepoData};
use crate::modules::repo::PackageMetaData;
use ipak::utils::color::colorize::*;
use crate::utils::www::*;
use std::fmt;
use std::str::FromStr;
use futures::future::join_all;
use tokio::runtime::Runtime; // tokio::runtime::Runtimeをインポート

// packages関数は同期関数として定義
pub fn packages() -> Result<Vec<PackageMetaData>, std::io::Error> {
    let rt = Runtime::new()?; // 新しいTokioランタイムを作成

    // block_onを使って、非同期処理を同期的に実行
    rt.block_on(async {
        let repos = get_indexes()?; // 同期的にリポジトリインデックスを取得

        let mut all_packages: Vec<PackageMetaData> = Vec::new();
        let mut fetch_futures = Vec::new();

        for repo_index in repos {
            let repo_type = repo_index.repo_type;
            // URL文字列をURL型に変換。失敗した場合はエラーを返す。
            let url = repo_index.url.to_url().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            
            // 各リポジトリのフェッチを非同期タスクとしてスポーン
            // `tokio::spawn`は新しいタスクをバックグラウンドで実行し、JoinHandleを返す
            fetch_futures.push(tokio::spawn(async move {
                // RepoData::new は非同期関数なのでawait
                RepoData::new(repo_type, url)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to fetch repo data: {}", e)))
            }));
        }

        // 全てのフェッチタスクが完了するのを待つ
        let results = join_all(fetch_futures).await; // 全てのスポーンされたタスクの完了を待つ

        for result in results {
            match result {
                Ok(Ok(repo_data)) => {
                    // 成功した場合、そのリポジトリのパッケージをall_packagesに追加
                    for package_meta_data in repo_data.packages {
                        all_packages.push(package_meta_data);
                    }
                },
                Ok(Err(e)) => {
                    eprintln!("Error fetching repository: {}", e);
                },
                Err(e) => {
                    // tokio::spawnで発生したパニックなど、タスク自体が失敗した場合
                    eprintln!("Task for fetching repository failed: {}", e);
                }
            }
        }

        Ok(all_packages)
    })
}

fn get_indexes() -> Result<Vec<RepoIndex>, std::io::Error> {
    let local_repos = path::local::repo_list_path();
    let global_repos = path::local::repo_list_path(); // Note: This currently points to the same path as local_repos.
    let local_content = std::fs::read_to_string(&local_repos).unwrap_or("".to_string());
    let global_content = std::fs::read_to_string(&global_repos).unwrap_or("".to_string()); // Reads from the same path if local_repos and global_repos are identical
    let mut repos = parse_repo(local_content)?;
    repos.extend(parse_repo(global_content)?);
    Ok(repos)
}
pub fn list() -> Result<(), std::io::Error> {
    let repos = get_indexes()?;
    for repo in repos {
        println!("{}", repo);
    }
    Ok(())
}
struct RepoIndex {
    repo_type: RepoType,
    url: String,
}
impl fmt::Display for RepoIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.repo_type.to_string().bold(),
            self.url.cyan()
        )
    }
}
fn parse_repo(
    s: String,
) -> Result<Vec<RepoIndex>, std::io::Error> {
    let mut result: Vec<RepoIndex> = Vec::new();
    let lines: Vec<&str> = s.split('\n').collect();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((repo_type, url)) = line.split_once(':') {
            result.push(RepoIndex {
                repo_type: RepoType::from_str(repo_type.trim())
                    .map_err(|e| -> std::io::Error {
                        std::io::Error::other(e)
                    })?,
                url: url.trim().to_string(),
            });
        } else {
            eprintln!("Warning: Malformed repository entry skipped: '{}'. Expected 'type:url' format.", line);
        }
    }
    Ok(result)
}