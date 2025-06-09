use super::super::RepoData;
use crate::utils::www::*;
use serde_yaml;
pub fn fetch(url: URL) -> Result<RepoData, std::io::Error> {
    let url = format!("{}/repo.yaml", url);
    let request = reqwest::blocking::get(url).map_err(
        |e| -> std::io::Error { std::io::Error::other(e) },
    )?;
    let request =
        request.text().map_err(|e| -> std::io::Error {
            std::io::Error::other(e)
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
