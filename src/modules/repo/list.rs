use crate::modules::{repo::RepoType, system::path};
use ipak::modules::pkg::PackageData;
use ipak::utils::color::colorize::*;
use std::fmt;
use std::str::FromStr;
pub fn packages() -> Result<Vec<PackageData>, std::io::Error> {

    Ok(Vec::new())
}
fn get() -> Result<Vec<RepoIndex>, std::io::Error> {
    let local_repos = path::local::repo_list_path();
    let global_repos = path::local::repo_list_path();
    let local_content = std::fs::read_to_string(&local_repos)?;
    let global_content = std::fs::read_to_string(&global_repos)?;
    let mut repos = parse_repo(local_content)?;
    repos.extend(parse_repo(global_content)?);
    Ok(repos)
}
pub fn list() -> Result<(), std::io::Error> {
    let repos = get()?;
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
    let lines: Vec<&str> = s.split("\n").collect();
    for line in lines {
        if let Some((repo_type, url)) = line.split_once(':') {
            result.push(RepoIndex {
                repo_type: RepoType::from_str(repo_type)
                    .map_err(|e| -> std::io::Error {
                        std::io::Error::other(e)
                    })?,
                url: url.to_string(),
            });
        }
    }
    Ok(result)
}
