use super::super::RepoData;
use crate::utils::www::*;
use serde_yaml;
pub fn fetch(url: URL) -> Result<RepoData, std::io::Error> {
    let url = url.join("repo.yaml")?;
    println!("{}", url);
    let request =
        url.fetch().map_err(|e| -> std::io::Error {
            std::io::Error::other(e.to_string())
        })?;
    let result: RepoData = serde_yaml::from_str(&request)
        .map_err(|e| -> std::io::Error {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?;
    Ok(result)
}
