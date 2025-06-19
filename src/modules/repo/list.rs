// src/modules/repo/list.rs
use crate::modules::repo::PackageMetaData;
use crate::modules::repo::RepoData;
use crate::modules::system::path;
use crate::utils::www::*;
use ipak::utils::color::colorize::*;
use std::fmt;

pub fn packages() -> Result<Vec<PackageMetaData>, std::io::Error>
{
    // 同期的にリポジトリインデックスを取得
    let repos = get_indexes()?;
    let mut all_packages = Vec::new();

    // 各リポジトリを同期的に処理
    for repo_index in repos {
        let url = repo_index.url.to_url().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e,
            )
        })?;

        // 同期的にリポジトリデータを取得
        match RepoData::new(repo_index.repo_type, url) {
            Ok(repo_data) => {
                all_packages.extend(repo_data.packages);
            }
            Err(e) => {
                eprintln!("Error fetching repository: {}", e);
            }
        }
    }

    Ok(all_packages)
}

fn get_indexes() -> Result<Vec<RepoIndex>, std::io::Error> {
    let local_repos = path::local::repo_list_path();
    let global_repos = path::global::repo_list_path(); // Note: This currently points to the same path as local_repos.
    let local_content = std::fs::read_to_string(&local_repos)
        .unwrap_or("".to_string());
    let global_content = std::fs::read_to_string(&global_repos)
        .unwrap_or("".to_string()); // Reads from the same path if local_repos and global_repos are identical
    let mut repos = parse_repo(local_content)?;
    repos.extend(parse_repo(global_content)?);
    Ok(repos)
}
pub fn list() -> Result<(), std::io::Error> {
    let repos = get_indexes()?;
    println!("{}:{}\n", "Total Repos".bold(), repos.len());
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
            "{}: {}",
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
            eprintln!(
                "Warning: Malformed repository entry skipped: '{}'. Expected 'type:url' format.",
                line
            );
        }
    }
    Ok(result)
}
