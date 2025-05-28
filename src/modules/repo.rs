mod apt;
struct RepoIndex {
    apt_info: Option<apt::AptInfo>,
}

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// PackageData、Version、VersionRangeは、以前提供されたコードで定義されているものとします。
// 例: use super::PackageData;
// 実際には、これらの構造体が定義されているモジュールへの適切なパスを指定してください。
// ここでは仮に `crate::package_data::` プレフィックスを使用します。
// 必要に応じて、`use` ステートメントを調整してください。
use ipak::modules::pkg::PackageData; // 仮のパス
use ipak::modules::version::{Version, VersionRange};
/// リポジトリのベースURLやアクセス情報を提供します。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    /// リポジトリのベースURL（例: "https://my-ipm-repo.example.com/"）
    pub base_url: String,
    // 将来的に認証情報やその他のリポジトリ設定を追加する可能性があります。
}

impl RepoInfo {
    /// 新しい `RepoInfo` インスタンスを作成します。
    ///
    /// # 引数
    /// * `base_url` - リポジトリのベースURL。
    ///
    /// # 戻り値
    /// 新しい `RepoInfo` インスタンス。
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// リポジトリのインデックスファイル（`index.json`）のURLを生成します。
    ///
    /// # 戻り値
    /// インデックスファイルの完全なURL文字列。
    pub fn index_url(&self) -> String {
        // ベースURLがスラッシュで終わることを確認し、適切に結合します。
        let mut url = self.base_url.trim_end_matches('/').to_string();
        url.push_str("/index.json");
        url
    }

    /// 特定のパッケージアーカイブのダウンロードURLを生成します。
    ///
    /// この関数は、`PackageData`のメタデータからダウンロードURLを構築します。
    ///
    /// # 引数
    /// * `package_name` - パッケージの名前。
    /// * `version` - パッケージのバージョン。
    /// * `architecture` - パッケージがビルドされたアーキテクチャ（例: "x86_64", "aarch64", "any"）。
    /// * `extension` - パッケージアーカイブのファイル拡張子（例: "zip", "tar.gz"）。
    ///
    /// # 戻り値
    /// パッケージアーカイブの完全なダウンロードURL文字列。
    pub fn package_archive_url(
        &self,
        package_name: &str,
        version: &Version,
        architecture: &str,
        extension: &str,
    ) -> String {
        let mut url = self.base_url.trim_end_matches('/').to_string();
        url.push_str(&format!(
            "/packages/{}/{}/{}-{}-{}.{}",
            package_name,
            version.to_string(), // Version型にDisplayトレイトが実装されていることを想定
            package_name,
            version.to_string(),
            architecture,
            extension
        ));
        url
    }
}

/// リポジトリに登録される個々のパッケージのメタデータと追加情報を表します。
/// これは、`PackageData` に加えて、ダウンロードに必要な情報を含みます。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoPackageData {
    /// パッケージの基本的な情報（名前、バージョン、作者、依存関係など）。
    /// `#[serde(flatten)]` により、`PackageData` のフィールドがこの構造体に直接展開されます。
    #[serde(flatten)]
    pub info: PackageData,
    /// このパッケージアーカイブの直接ダウンロードURL。
    pub download_url: String,
    /// パッケージアーカイブのファイルサイズ（バイト単位）。
    pub data_size: u64,
    /// パッケージアーカイブのSHA256チェックサム文字列。
    /// ダウンロード後のファイルの整合性検証に使用されます。
    pub checksum_sha256: String,
    /// リポジトリに登録された、または最後に更新された日時。
    pub last_updated: DateTime<Local>,
}

impl RepoPackageData {
    /// 新しい `RepoPackageData` インスタンスを作成します。
    ///
    /// # 引数
    /// * `info` - パッケージの基本情報 (`PackageData` 構造体)。
    /// * `download_url` - パッケージアーカイブのダウンロードURL。
    /// * `data_size` - パッケージアーカイブのサイズ（バイト単位）。
    /// * `checksum_sha256` - パッケージアーカイブのSHA256チェックサム。
    ///
    /// # 戻り値
    /// 新しい `RepoPackageData` インスタンス。
    pub fn new(
        info: PackageData,
        download_url: String,
        data_size: u64,
        checksum_sha256: String,
    ) -> Self {
        Self {
            info,
            download_url,
            data_size,
            checksum_sha256,
            last_updated: Local::now(), // 作成時に現在時刻を設定
        }
    }
}

// テスト用のダミーの PackageData, Version, VersionRange の定義
// 実際のプロジェクトでは、これらの定義は別のファイルにあるはずです。
// この `mod` ブロックは、このファイル単体でコンパイルテストを行うためのものです。
#[cfg(test)]
mod tests {
    pub mod package_data {
        use serde::{Deserialize, Serialize};
        use std::fmt::{self, Display, Formatter};
        use std::str::FromStr;

        #[derive(
            Debug, Clone, Serialize, Deserialize, PartialEq, Default,
        )]
        pub enum Mode {
            Local,
            Global,
            #[default]
            Any,
        }

        impl Display for Mode {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    Mode::Local => write!(f, "local"),
                    Mode::Global => write!(f, "global"),
                    Mode::Any => write!(f, "any (local & global)"),
                }
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct PackageData {
            pub about: AboutData,
            pub architecture: Vec<String>,
            pub mode: Mode,
            pub relation: RelationData,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct AboutData {
            pub author: AuthorAboutData,
            pub package: PackageAboutData,
        }

        #[derive(Debug, Serialize, Deserialize, Clone)]
        #[serde(default)]
        pub struct AuthorAboutData {
            pub name: String,
            pub email: String,
        }

        impl Default for AuthorAboutData {
            fn default() -> Self {
                AuthorAboutData {
                    name: "Default Author".to_string(),
                    email: "default@example.com".to_string(),
                }
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(default)]
        pub struct PackageAboutData {
            pub name: String,
            pub version: Version,
        }

        impl Default for PackageAboutData {
            fn default() -> Self {
                PackageAboutData {
                    name: "default-package".to_string(),
                    version: Version::default(),
                }
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct RelationData {
            pub depend: Vec<Vec<PackageRange>>,
            pub depend_cmds: Vec<String>,
            pub suggests: Vec<Vec<PackageRange>>,
            pub recommends: Vec<Vec<PackageRange>>,
            pub conflicts: Vec<PackageRange>,
            pub virtuals: Vec<PackageVersion>,
            pub provide_cmds: Vec<String>,
        }

        impl RelationData {
            pub fn is_empty(&self) -> bool {
                self.depend.is_empty()
                    && self.depend_cmds.is_empty()
                    && self.suggests.is_empty()
                    && self.recommends.is_empty()
                    && self.conflicts.is_empty()
                    && self.virtuals.is_empty()
                    && self.provide_cmds.is_empty()
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(default)]
        pub struct PackageRange {
            pub name: String,
            pub range: VersionRange,
        }

        impl Default for PackageRange {
            fn default() -> Self {
                PackageRange {
                    name: "default-dependency".to_string(),
                    range: VersionRange::default(),
                }
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(default)]
        pub struct PackageVersion {
            pub name: String,
            pub version: Version,
        }

        impl Default for PackageVersion {
            fn default() -> Self {
                PackageVersion {
                    name: "default-version".to_string(),
                    version: Version::default(),
                }
            }
        }

        // ダミーの Version 構造体 (セマンティックバージョニングを模倣)
        #[derive(
            Debug,
            Clone,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
        )]
        pub struct Version {
            pub major: u32,
            pub minor: u32,
            pub patch: u32,
            pub pre: Option<String>,
            pub build: Option<String>,
        }

        impl Default for Version {
            fn default() -> Self {
                Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                    pre: None,
                    build: None,
                }
            }
        }

        impl Display for Version {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
                if let Some(ref pre) = self.pre {
                    write!(f, "-{}", pre)?;
                }
                if let Some(ref build) = self.build {
                    write!(f, "+{}", build)?;
                }
                Ok(())
            }
        }

        impl FromStr for Version {
            type Err = String; // 簡単のためStringエラー
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let parts: Vec<&str> = s.split('.').collect();
                if parts.len() < 3 {
                    return Err("Invalid version format".to_string());
                }
                let major = parts[0]
                    .parse()
                    .map_err(|_| "Invalid major version".to_string())?;
                let minor = parts[1]
                    .parse()
                    .map_err(|_| "Invalid minor version".to_string())?;
                let patch_and_rest: Vec<&str> =
                    parts[2].splitn(2, '-').collect();
                let patch = patch_and_rest[0]
                    .parse()
                    .map_err(|_| "Invalid patch version".to_string())?;

                let pre = if patch_and_rest.len() > 1 {
                    let pre_and_build: Vec<&str> =
                        patch_and_rest[1].splitn(2, '+').collect();
                    Some(pre_and_build[0].to_string())
                } else {
                    None
                };

                let build = if let Some(ref p) = pre {
                    let pre_and_build: Vec<&str> =
                        patch_and_rest[1].splitn(2, '+').collect();
                    if pre_and_build.len() > 1 {
                        Some(pre_and_build[1].to_string())
                    } else {
                        None
                    }
                } else if parts[2].contains('+') {
                    let build_parts: Vec<&str> =
                        parts[2].splitn(2, '+').collect();
                    if build_parts.len() > 1 {
                        Some(build_parts[1].to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Version { major, minor, patch, pre, build })
            }
        }

        // ダミーの VersionRange 構造体
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(untagged)] // Enum variants without tags
        pub enum VersionRange {
            Exact(Version),
            GreaterThan(Version),
            GreaterThanOrEqual(Version),
            LessThan(Version),
            LessThanOrEqual(Version),
            WildcardPatch(u32, u32), // e.g., 1.2.x
            WildcardMinor(u32),      // e.g., 1.x.x
            Any,
        }

        impl Default for VersionRange {
            fn default() -> Self {
                VersionRange::Any
            }
        }

        impl Display for VersionRange {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    VersionRange::Exact(v) => write!(f, "={}", v),
                    VersionRange::GreaterThan(v) => write!(f, ">{}", v),
                    VersionRange::GreaterThanOrEqual(v) => {
                        write!(f, ">={}", v)
                    }
                    VersionRange::LessThan(v) => write!(f, "<{}", v),
                    VersionRange::LessThanOrEqual(v) => {
                        write!(f, "<={}", v)
                    }
                    VersionRange::WildcardPatch(major, minor) => {
                        write!(f, "{}.{}.x", major, minor)
                    }
                    VersionRange::WildcardMinor(major) => {
                        write!(f, "{}.x.x", major)
                    }
                    VersionRange::Any => write!(f, "*"),
                }
            }
        }

        impl FromStr for VersionRange {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s == "*" {
                    return Ok(VersionRange::Any);
                }
                if s.starts_with(">=") {
                    return Ok(VersionRange::GreaterThanOrEqual(
                        Version::from_str(&s[2..])?,
                    ));
                }
                if s.starts_with(">") {
                    return Ok(VersionRange::GreaterThan(
                        Version::from_str(&s[1..])?,
                    ));
                }
                if s.starts_with("<=") {
                    return Ok(VersionRange::LessThanOrEqual(
                        Version::from_str(&s[2..])?,
                    ));
                }
                if s.starts_with("<") {
                    return Ok(VersionRange::LessThan(Version::from_str(
                        &s[1..],
                    )?));
                }
                if s.starts_with("=") {
                    return Ok(VersionRange::Exact(Version::from_str(
                        &s[1..],
                    )?));
                }
                if s.ends_with(".x") {
                    let parts: Vec<&str> =
                        s.trim_end_matches(".x").split('.').collect();
                    if parts.len() == 2 {
                        let major = parts[0].parse().map_err(|_| {
                            "Invalid major for wildcard".to_string()
                        })?;
                        let minor = parts[1].parse().map_err(|_| {
                            "Invalid minor for wildcard".to_string()
                        })?;
                        return Ok(VersionRange::WildcardPatch(
                            major, minor,
                        ));
                    } else if parts.len() == 1 {
                        let major = parts[0].parse().map_err(|_| {
                            "Invalid major for wildcard".to_string()
                        })?;
                        return Ok(VersionRange::WildcardMinor(major));
                    }
                }
                // デフォルトはExactマッチング
                Ok(VersionRange::Exact(Version::from_str(s)?))
            }
        }
    }
}

#[cfg(test)]
mod test2 {
    use super::*;
    use ipak::modules::pkg::{AboutData, AuthorAboutData, PackageData,PackageAboutData};
    use ipak::modules::pkg::{Mode,RelationData};
    use ipak::modules::version::{Version, VersionRange};
    use std::str::FromStr;

    #[test]
    fn test_repo_info_index_url() {
        let repo_info =
            RepoInfo::new("https://example.com/repo".to_string());
        assert_eq!(
            repo_info.index_url(),
            "https://example.com/repo/index.json"
        );

        let repo_info_trailing_slash =
            RepoInfo::new("https://example.com/repo/".to_string());
        assert_eq!(
            repo_info_trailing_slash.index_url(),
            "https://example.com/repo/index.json"
        );
    }

    #[test]
    fn test_repo_info_package_archive_url() {
        let repo_info =
            RepoInfo::new("https://example.com/repo".to_string());
        let version = Version::from_str("1.0.0").unwrap();
        let url = repo_info
            .package_archive_url("my-app", &version, "x86_64", "zip");
        assert_eq!(
            url,
            "https://example.com/repo/packages/my-app/1.2.3/my-app-1.2.3-x86_64.zip"
        );

        let version_pre = Version::from_str("1.2.3-beta").unwrap();
        let url_pre = repo_info.package_archive_url(
            "my-app",
            &version_pre,
            "aarch64",
            "tar.gz",
        );
        assert_eq!(
            url_pre,
            "https://example.com/repo/packages/my-app/1.2.3-beta/my-app-1.2.3-beta-aarch64.tar.gz"
        );
    }

    #[test]
    fn test_repo_package_data_new() {
        let package_info = PackageData {
            about: AboutData {
                author: AuthorAboutData {
                    name: "Test Author".to_string(),
                    email: "test@example.com".to_string(),
                },
                package: PackageAboutData {
                    name: "test-package".to_string(),
                    version: Version::from_str("1.0.0").unwrap(),
                },
            },
            architecture: vec!["x86_64".to_string()],
            mode: Mode::Global,
            relation: RelationData::default(),
        };

        let download_url =
            "https://example.com/pkg/test-package-1.0.0.zip".to_string();
        let data_size = 1024 * 1024; // 1MB
        let checksum = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2".to_string();

        let repo_pkg_data = RepoPackageData::new(
            package_info.clone(),
            download_url.clone(),
            data_size,
            checksum.clone(),
        );

        assert_eq!(repo_pkg_data.info.about.package.name, "test-package");
        assert_eq!(
            repo_pkg_data.info.about.package.version,
            Version::from_str("1.0.0").unwrap()
        );
        assert_eq!(repo_pkg_data.download_url, download_url);
        assert_eq!(repo_pkg_data.data_size, data_size);
        assert_eq!(repo_pkg_data.checksum_sha256, checksum);
        // last_updated は現在時刻に近いことを確認
        let now = Local::now();
        assert!(repo_pkg_data.last_updated <= now);
        assert!(
            repo_pkg_data.last_updated
                > now - chrono::Duration::seconds(5)
        );
    }

    #[test]
    fn test_repo_package_data_serialization_deserialization() {
        let package_info = PackageData {
            about: AboutData {
                author: AuthorAboutData {
                    name: "Serializer Test".to_string(),
                    email: "ser@example.com".to_string(),
                },
                package: PackageAboutData {
                    name: "serial-pkg".to_string(),
                    version: Version::from_str("1.1.0").unwrap(),
                },
            },
            architecture: vec![
                "x86_64".to_string(),
                "aarch64".to_string(),
            ],
            mode: Mode::Local,
            relation: RelationData::default(),
        };

        let repo_pkg_data = RepoPackageData::new(
            package_info,
            "https://example.com/pkg/serial-pkg-1.1.0.tar.gz".to_string(),
            2048,
            "c0ffee".to_string(),
        );

        let serialized = serde_json::to_string(&repo_pkg_data).unwrap();
        println!("Serialized: {}", serialized);

        let deserialized: RepoPackageData =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.info.about.package.name, "serial-pkg");
        assert_eq!(
            deserialized.download_url,
            "https://example.com/pkg/serial-pkg-1.1.0.tar.gz"
        );
        assert_eq!(deserialized.data_size, 2048);
        assert_eq!(deserialized.checksum_sha256, "c0ffee");
        assert_eq!(
            deserialized.info.architecture,
            vec!["x86_64".to_string(), "aarch64".to_string()]
        );
        assert_eq!(deserialized.info.mode, Mode::Local);
    }
}
