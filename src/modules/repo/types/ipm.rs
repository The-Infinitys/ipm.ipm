use super::super::RepoData;
use crate::utils::www::*;
use serde_yaml;
pub fn fetch(url: URL) -> Result<RepoData, std::io::Error> {
    let url = url.clone().join("repo.yaml")?;
    println!("{}", url);
    let request =
        url.fetch().map_err(|e| -> std::io::Error {
            std::io::Error::other(e.to_string())
        })?;
    let mut result: RepoData = serde_yaml::from_str(&request)
        .map_err(|e| -> std::io::Error {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?;
    for pkg in &mut result.packages {
        pkg.url = url.clone().join(&pkg.url)?.to_string()
    }
    Ok(result)
}
