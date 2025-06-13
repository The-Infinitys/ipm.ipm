use super::super::{PackageMetaData, RepoData};
use crate::utils::www::URL;
use anyhow::{Result, anyhow};
use chrono::Local;
use flate2::read::GzDecoder;
use ipak::modules::{
    pkg::{
        AboutData, AuthorAboutData, Mode, PackageAboutData,
        PackageData, PackageRange, PackageVersion, RelationData,
    },
    version::{Version, VersionRange},
};
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, str::FromStr};

/// APTパッケージのcontrolファイルを解析し、HashMapに変換します。
pub fn parse_control_file(
    control_content: &str,
) -> Result<HashMap<String, String>> {
    let mut data: HashMap<String, String> = HashMap::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut is_multiline = false;

    for line in control_content.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // マルチラインの値の続き
            if is_multiline {
                current_value.push('\n');
                current_value.push_str(line.trim());
            }
            continue;
        }
        // 新しいキー:値 のペアが始まる場合、前のものを保存
        if !current_key.is_empty() {
            data.insert(
                current_key.clone(),
                current_value.clone(),
            );
        }
        if let Some(index) = line.find(':') {
            current_key = line[..index].trim().to_string();
            current_value = line[index + 1..].trim().to_string();
            is_multiline = current_value.is_empty();
        } else {
            current_key.clear();
            current_value.clear();
            is_multiline = false;
        }
    }
    // 最後のキー:値 を保存
    if !current_key.is_empty() {
        data.insert(current_key, current_value);
    }

    Ok(data)
}

/// 解析されたHashMapからPackageData構造体を生成します。
pub fn to_package_data(
    data: HashMap<String, String>,
) -> Result<PackageData> {
    let package_name = data
        .get("Package")
        .ok_or_else(|| {
            anyhow!("'Package' field not found in control file")
        })?
        .to_string();
    let version_str = data.get("Version").ok_or_else(|| {
        anyhow!("'Version' field not found in control file")
    })?;
    let version =
        Version::from_str(version_str).map_err(|e| {
            anyhow!(
                "Failed to parse version '{}': {}",
                version_str,
                e
            )
        })?;
    let description = data
        .get("Description")
        .unwrap_or(&String::new())
        .to_string();
    let architecture = data
        .get("Architecture")
        .map(|s| {
            s.split(',')
                .map(|arch| arch.trim().to_string())
                .collect()
        })
        .unwrap_or_default();
    let empty_string = String::new();
    let maintainer_str =
        data.get("Maintainer").unwrap_or(&empty_string);
    let (author_name, author_email) =
        parse_maintainer(maintainer_str);
    let author_data = AuthorAboutData {
        name: author_name,
        email: author_email,
    };

    let mut relation_data = RelationData::default();

    // Dependencies (Depends)
    if let Some(depends_str) = data.get("Depends") {
        relation_data.depend =
            parse_package_ranges(depends_str)?;
    }
    // Suggests
    if let Some(suggests_str) = data.get("Suggests") {
        relation_data.suggests =
            parse_package_ranges(suggests_str)?;
    }
    // Recommends
    if let Some(recommends_str) = data.get("Recommends") {
        relation_data.recommends =
            parse_package_ranges(recommends_str)?;
    }
    // Conflicts
    if let Some(conflicts_str) = data.get("Conflicts") {
        relation_data.conflicts =
            parse_single_package_ranges(conflicts_str)?;
    }
    // Provides (Virtual Packages)
    if let Some(provides_str) = data.get("Provides") {
        relation_data.virtuals =
            parse_package_versions(provides_str)?;
    }

    Ok(PackageData {
        about: AboutData {
            author: author_data,
            package: PackageAboutData {
                name: package_name,
                version,
                description,
            },
        },
        architecture,
        mode: Mode::Global, // APTパッケージは通常、システム単位でパッケージをインストールするため、Global
        relation: relation_data,
    })
}

/// Maintainer文字列から名前とメールアドレスをパースします。
/// 例: "John Doe <john.doe@example.com>"
fn parse_maintainer(maintainer_str: &str) -> (String, String) {
    if let Some(start) = maintainer_str.find('<') {
        if let Some(end) = maintainer_str.find('>') {
            let name =
                maintainer_str[..start].trim().to_string();
            let email = maintainer_str[start + 1..end]
                .trim()
                .to_string();
            return (name, email);
        }
    }
    // メールアドレスがない場合
    (maintainer_str.to_string(), String::new())
}

/// 依存関係の文字列（例: "pkg-a (>= 1.0) | pkg-b, pkg-c (<< 2.0)"）をパースします。
fn parse_package_ranges(
    input: &str,
) -> Result<Vec<Vec<PackageRange>>> {
    let mut result = Vec::new();
    for group_str in input.split(',') {
        let alternatives: Result<Vec<PackageRange>> = group_str
            .split('|')
            .map(|alt_str| {
                let parts: Vec<&str> = alt_str.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(anyhow!("Empty package range alternative"));
                }
                let name = parts[0].to_string();
                let range = if parts.len() > 1 {
                    let version_range_str = parts[1..].join(" ");
                    let version_range_str = version_range_str
                        .trim_start_matches('(')
                        .trim_end_matches(')');
                    VersionRange::from_str(version_range_str).map_err(|e| {
                        anyhow!(
                            "Failed to parse version range '{}' for package '{}': {}",
                            version_range_str,
                            name,
                            e
                        )
                    })?
                } else {
                    VersionRange::default()
                };
                Ok(PackageRange { name, range })
            })
            .collect();
        result.push(alternatives?);
    }
    Ok(result)
}

/// 単一の依存関係の文字列（例: "pkg-a (>= 1.0), pkg-b"）をパースします。
fn parse_single_package_ranges(
    input: &str,
) -> Result<Vec<PackageRange>> {
    input
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.is_empty() {
                return Err(anyhow!("Empty package range"));
            }
            let name = parts[0].to_string();
            let range = if parts.len() > 1 {
                let version_range_str = parts[1..].join(" ");
                let version_range_str = version_range_str
                    .trim_start_matches('(')
                    .trim_end_matches(')');
                VersionRange::from_str(version_range_str).map_err(|e| {
                    anyhow!(
                        "Failed to parse version range '{}' for package '{}': {}",
                        version_range_str,
                        name,
                        e
                    )
                })?
            } else {
                VersionRange::default()
            };
            Ok(PackageRange { name, range })
        })
        .collect()
}

/// virtualパッケージの文字列（例: "pkg-virtual (1.0), another-virtual"）をパースします。
fn parse_package_versions(
    input: &str,
) -> Result<Vec<PackageVersion>> {
    input
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.is_empty() {
                return Err(anyhow!("Empty package version"));
            }
            let name = parts[0].to_string();
            let version = if parts.len() > 1 {
                let version_str = parts[1..].join(" ");
                let version_str = version_str
                    .trim_start_matches('(')
                    .trim_end_matches(')');
                Version::from_str(version_str).map_err(|e| {
                    anyhow!(
                        "Failed to parse version '{}' for package '{}': {}",
                        version_str,
                        name,
                        e
                    )
                })?
            } else {
                Version::default()
            };
            Ok(PackageVersion { name, version })
        })
        .collect()
}

/// Helper function to extract filename from control content.
/// This is a simplified version and might need refinement based on actual control file content.
fn get_filename(control_content: &str) -> String {
    let mut filename = String::new();
    for line in control_content.lines() {
        if let Some(stripped) = line.strip_prefix("Filename:") {
            filename = stripped.trim().to_string();
            break;
        }
    }
    filename
}

/// 指定されたURLからPackages.gzファイルをダウンロードし、解析してRepoDataを返します。
pub fn fetch(url: URL) -> Result<RepoData, std::io::Error> {
    // URLに"Packages.gz"を結合
    let packages_url =
        url.clone().join("Packages.gz").map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?;

    // HTTPリクエストでPackages.gzをダウンロード
    let response_text =
        packages_url.fetch_bin().map_err(|e| {
            std::io::Error::other(
                format!("Failed to fetch Packages.gz: {}", e),
            )
        })?;

    // Gzipを解凍
    let compressed_data = &response_text[..];
    let decoder = GzDecoder::new(compressed_data);
    let reader = BufReader::new(decoder);

    // パッケージデータを格納するベクター
    let mut packages = Vec::new();
    let mut current_control = String::new();

    // Packagesファイルは複数のcontrolエントリが空行で区切られている
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            // 空行の場合、現在のcontrolデータを解析
            if !current_control.is_empty() {
                match parse_control_file(&current_control) {
                    Ok(parsed_data_map) => {
                        match to_package_data(parsed_data_map) {
                            Ok(package_data) => {
                                let package_url_str =
                                    get_filename(
                                        &current_control,
                                    );
                                // urlのpathの最初のディレクトリを取得
                                let parent_path = url
                                    .path()
                                    .to_str()
                                    .unwrap()
                                    .to_owned();
                                let parent_path = parent_path
                                    .trim_start_matches('/')
                                    .split('/')
                                    .next()
                                    .unwrap_or("");
                                let package_url = url.clone()
                                    .join(format!("/{}/{}",parent_path,package_url_str).as_str())
                                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                                packages.push(PackageMetaData {
                                    last_modified: Local::now(), // 実際の値は取得できないためデフォルト
                                    info: package_data,
                                    url: package_url.to_string(),
                                });
                            }
                            Err(e) => eprintln!(
                                "Failed to convert HashMap to PackageData: {}",
                                e
                            ),
                        }
                    }
                    Err(e) => eprintln!(
                        "Failed to parse control block into HashMap: {}",
                        e
                    ),
                }
                current_control.clear();
            }
        } else {
            // 行をcurrent_controlに追加
            current_control.push_str(&line);
            current_control.push('\n');
        }
    }

    Ok(RepoData {
        author: AuthorAboutData {
            name: "Debian Repository".to_string(),
            email: "debian@debian.org".to_string(),
        },
        last_modified: Local::now(),
        packages,
    })
}
