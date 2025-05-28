use colored::Colorize;
use std::{fmt, str::FromStr};
use regex::Regex;
use serde_yaml;
use ipak::modules::pkg::list::PackageListData;

pub struct IpmInfo {
    url: String,
}

impl fmt::Display for IpmInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}-> {}",
            "IPM Repository".bold().green(),
            "URL".bold(),
            self.url
        )
    }
}

impl FromStr for IpmInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").map_err(|e| e.to_string())?;
        if url_regex.is_match(s) {
            Ok(IpmInfo { url: s.to_string() })
        } else {
            Err(format!("Invalid URL format: {}", s))
        }
    }
}

impl IpmInfo {
    pub async fn get_packages(&self) -> Result<PackageListData, Box<dyn std::error::Error>> {
        let packages_url = format!("{}/packages.yaml", self.url);
        let response = reqwest::get(&packages_url).await?;
        let package_list_data= response.text().await?;
        let package_list_data:PackageListData = serde_yaml::from_str(&package_list_data)?;
        Ok(package_list_data)
    }
}
