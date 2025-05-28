use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use colored::*;
use deb822_lossless::Deb822;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::io::Read;
use std::str::FromStr;
#[derive(Default)]
pub enum AptType {
    #[default]
    Deb,
    DebSrc,
}

impl fmt::Display for AptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AptType::Deb => write!(f, "{}", "deb".cyan()),
            AptType::DebSrc => write!(f, "{}", "deb-src".cyan()),
        }
    }
}

pub struct AptInfo {
    types: AptType,
    uris: String,
    suites: Vec<String>,
    components: Vec<String>,
    architectures: Vec<String>,
}

impl fmt::Display for AptInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type: {}\nURI: {}\nSuites: {}\nComponents: {}\nArchitectures: {}",
            self.types,
            self.uris.green(),
            self.suites.join(", ").yellow(),
            self.components.join(", ").magenta(),
            self.architectures.join(", ").blue()
        )
    }
}

pub struct Package {
    suite: String,
    component: String,
    architecture: Option<String>,
    fields: HashMap<String, String>,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.fields.get("Package").map_or("N/A".to_string(), |s| s.to_string());
        let version = self.fields.get("Version").map_or("N/A".to_string(), |s| s.to_string());
        let arch = self.architecture.as_ref().map_or("N/A".to_string(), |s| s.to_string());
        write!(
            f,
            "Package: {}\nVersion: {}\nSuite: {}\nComponent: {}\nArchitecture: {}",
            name.red().bold(),
            version.green(),
            self.suite.yellow(),
            self.component.magenta(),
            arch.blue()
        )
    }
}

pub async fn fetch_package_indices(apt_info: &AptInfo) -> Result<Vec<Package>, Box<dyn Error>> {
    let client = Client::new();
    let mut packages = Vec::new();
    let uri = &apt_info.uris;

    match apt_info.types {
        AptType::Deb => {
            for suite in &apt_info.suites {
                for component in &apt_info.components {
                    for arch in &apt_info.architectures {
                        // 圧縮ファイルと非圧縮ファイルの両方を試す
                        let urls = vec![
                            format!("{}/dists/{}/{}/binary-{}/Packages.gz", uri, suite, component, arch),
                            format!("{}/dists/{}/{}/binary-{}/Packages", uri, suite, component, arch),
                        ];
                        let mut content = None;
                        for url in urls {
                            match fetch_url(&client, &url).await {
                                Ok(data) => {
                                    content = Some(data);
                                    break;
                                }
                                Err(_) => continue,
                            }
                        }
                        let content = content.ok_or_else(|| format!("Failed to fetch Packages for {suite}/{component}/{arch}"))?;
                        let deb822 = Deb822::from_str(&content)?;
                        for para in deb822.paragraphs() {
                            let mut fields = HashMap::new();
                            for (key, value) in para.items() {
                                fields.insert(key.to_string(), value.to_string());
                            }
                            let package = Package {
                                suite: suite.clone(),
                                component: component.clone(),
                                architecture: Some(arch.clone()),
                                fields,
                            };
                            packages.push(package);
                        }
                    }
                }
            }
        }
        AptType::DebSrc => {
            for suite in &apt_info.suites {
                for component in &apt_info.components {
                    // 圧縮ファイルと非圧縮ファイルの両方を試す
                    let urls = vec![
                        format!("{}/dists/{}/{}/source/Sources.gz", uri, suite, component),
                        format!("{}/dists/{}/{}/source/Sources", uri, suite, component),
                    ];
                    let mut content = None;
                    for url in urls {
                        match fetch_url(&client, &url).await {
                            Ok(data) => {
                                content = Some(data);
                                break;
                            }
                            Err(_) => continue,
                        }
                    }
                    let content = content.ok_or_else(|| format!("Failed to fetch Sources for {suite}/{component}"))?;
                    let deb822 = Deb822::from_str(&content)?;
                    for para in deb822.paragraphs() {
                        let mut fields = HashMap::new();
                        for (key, value) in para.items() {
                            fields.insert(key.to_string(), value.to_string());
                        }
                        let package = Package {
                            suite: suite.clone(),
                            component: component.clone(),
                            architecture: None,
                            fields,
                        };
                        packages.push(package);
                    }
                }
            }
        }
    }

    Ok(packages)
}

async fn fetch_url(client: &Client, url: &str) -> Result<String, Box<dyn Error>> {
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {} for URL: {}", response.status(), url).into());
    }
    let bytes = response.bytes().await?;
    
    // .gzで終わる場合は圧縮を解凍
    if url.ends_with(".gz") {
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        Ok(content)
    } else {
        Ok(String::from_utf8(bytes.to_vec())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_noble_packages() {
        let apt_info = AptInfo {
            types: AptType::Deb,
            uris: "http://archive.ubuntu.com/ubuntu".to_string(),
            suites: vec!["noble".to_string()],
            components: vec!["main".to_string()],
            architectures: vec!["amd64".to_string()],
        };

        println!("Repository Info:\n{}", apt_info);

        match fetch_package_indices(&apt_info).await {
            Ok(packages) => {
                for package in packages.iter().take(5) { // 最初の5パッケージを表示
                    println!("\n{}", package);
                }
                println!("\nTotal packages: {}", packages.len());
                assert!(!packages.is_empty(), "No packages were fetched");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("Failed to fetch packages: {}", e);
            }
        }
    }
}