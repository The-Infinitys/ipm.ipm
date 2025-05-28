use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use colored::*;
use deb822_lossless::Deb822;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::io::Read;
use std::str::FromStr;
use std::io;

// ipak::modules::pkgからのインポート
// 実際にはクレートの構造に合わせてパスを調整してください
use ipak::modules::pkg::{PackageData, AboutData, AuthorAboutData, PackageAboutData, RelationData, PackageRange, PackageVersion};
use ipak::modules::version::{Version, VersionRange}; // ipak の Version と VersionRange

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
    pub types: AptType,
    pub uris: String,
    pub suites: Vec<String>,
    pub components: Vec<String>,
    pub architectures: Vec<String>,
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

impl AptInfo {
    /// 指定された AptInfo に基づいてパッケージインデックスを取得します。
    pub async fn get_packages(&self) -> Result<Vec<Package>, io::Error> {
        let client = Client::new();
        let mut packages = Vec::new();
        let uri = &self.uris;

        match self.types {
            AptType::Deb => {
                for suite in &self.suites {
                    for component in &self.components {
                        for arch in &self.architectures {
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
                                    Err(_) => continue, // 圧縮/非圧縮のどちらかが失敗してももう一方を試す
                                }
                            }
                            let content = content
                                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Failed to fetch Packages for {suite}/{component}/{arch}")))?;
                            
                            let deb822 = Deb822::from_str(&content)
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse Deb822 format: {}", e)))?;
                            
                            for para in deb822.paragraphs() {
                                let mut fields_map = HashMap::new();
                                for (key, value) in para.items() {
                                    fields_map.insert(key.to_string(), value.to_string());
                                }
                                match Package::try_from_deb822_paragraph(
                                    fields_map,
                                    suite.clone(),
                                    component.clone(),
                                    Some(arch.clone()),
                                ) {
                                    Ok(package) => packages.push(package),
                                    Err(e) => eprintln!("Warning: Skipping package due to parsing error: {}", e), // パースエラーはスキップ
                                }
                            }
                        }
                    }
                }
            }
            AptType::DebSrc => {
                for suite in &self.suites {
                    for component in &self.components {
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
                        let content = content
                            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Failed to fetch Sources for {suite}/{component}")))?;
                        
                        let deb822 = Deb822::from_str(&content)
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse Deb822 format: {}", e)))?;
                        
                        for para in deb822.paragraphs() {
                            let mut fields_map = HashMap::new();
                            for (key, value) in para.items() {
                                fields_map.insert(key.to_string(), value.to_string());
                            }
                            match Package::try_from_deb822_paragraph(
                                fields_map,
                                suite.clone(),
                                component.clone(),
                                None, // No architecture for deb-src
                            ) {
                                Ok(package) => packages.push(package),
                                Err(e) => eprintln!("Warning: Skipping source package due to parsing error: {}", e),
                            }
                        }
                    }
                }
            }
        }

        Ok(packages)
    }
}


/// Represents a single package entry as defined in Debian Policy Manual.
/// Common fields are parsed into their own fields for easier access and type safety.
/// All other fields are stored in a HashMap.
#[derive(Debug)] // Debug トレイトを追加
pub struct Package {
    // Mandatory fields
    pub package: String, // 'Package' field
    pub version: String, // 'Version' field (Debian policy version string)
    pub maintainer: String, // 'Maintainer' field

    // Description is not mandatory for binary packages, but is for source packages (debian/control)
    pub description: Option<String>, // 'Description' field

    // Fields relevant to the package index file context
    pub suite: String,
    pub component: String,
    pub architecture: Option<String>, // 'Architecture' field, mandatory for deb, not present for deb-src

    // Optional fields, now using PackageRange and PackageVersion for dependencies
    pub section: Option<String>, // 'Section' field
    pub priority: Option<String>, // 'Priority' field
    pub homepage: Option<String>, // 'Homepage' field
    pub depends: Vec<Vec<PackageRange>>, // 'Depends' field, parsed into nested vectors for OR relations
    pub recommends: Vec<Vec<PackageRange>>, // 'Recommends' field
    pub suggests: Vec<Vec<PackageRange>>, // 'Suggests' field
    pub conflicts: Vec<PackageRange>, // 'Conflicts' field
    pub replaces: Vec<PackageRange>, // 'Replaces' field
    pub provides: Vec<PackageVersion>, // 'Provides' field
    pub built_using: Option<String>, // 'Built-Using' field (for deb-src)
    pub original_maintainer: Option<String>, // 'Original-Maintainer' field (for deb-src)


    // All other fields not explicitly parsed above
    pub other_fields: HashMap<String, String>,
}

impl Package {
    /// Parses a single dependency string (e.g., "pkg1 (>= 1.0) | pkg2 (= 2.0)")
    /// into a vector of PackageRange vectors.
    /// Each inner vector represents an OR group.
    fn parse_dependencies(dep_str: &str) -> Vec<Vec<PackageRange>> {
        dep_str.split(',')
            .map(|group_str| {
                group_str.split('|')
                    .map(|dep_part| {
                        let trimmed_dep = dep_part.trim();
                        // Example: "pkg-name (>= 1.0)" or "pkg-name"
                        let parts: Vec<&str> = trimmed_dep.splitn(2, ' ').collect();
                        let name = parts[0].to_string();
                        let range = if parts.len() > 1 {
                            // Version range exists, e.g., "(>= 1.0)"
                            let version_range_str = parts[1].trim_matches(|c| c == '(' || c == ')');
                            VersionRange::from_str(version_range_str).unwrap_or_default()
                        } else {
                            // No version range specified, assume any version
                            VersionRange::default() // Represents any version
                        };
                        PackageRange { name, range }
                    })
                    .collect()
            })
            .collect()
    }

    /// Parses a single Provides string (e.g., "virtual-pkg (= 1.0), another-virtual")
    /// into a vector of PackageVersion.
    fn parse_provides(provides_str: &str) -> Vec<PackageVersion> {
        provides_str.split(',')
            .map(|provide_part| {
                let trimmed_provide = provide_part.trim();
                let parts: Vec<&str> = trimmed_provide.splitn(2, ' ').collect();
                let name = parts[0].to_string();
                let version = if parts.len() > 1 {
                    let version_str = parts[1].trim_matches(|c| c == '(' || c == ')');
                    Version::from_str(version_str).unwrap_or_default()
                } else {
                    Version::default() // No version specified, use default
                };
                PackageVersion { name, version }
            })
            .collect()
    }


    /// Tries to create a `Package` from a `HashMap<String, String>` representing a Deb822 paragraph.
    /// Returns an error if mandatory fields are missing.
    pub fn try_from_deb822_paragraph(
        mut fields: HashMap<String, String>,
        suite: String,
        component: String,
        architecture: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        let package = fields.remove("Package")
            .ok_or_else(|| "Missing mandatory 'Package' field".to_string())?;
        let version = fields.remove("Version")
            .ok_or_else(|| "Missing mandatory 'Version' field".to_string())?;
        let maintainer = fields.remove("Maintainer")
            .ok_or_else(|| "Missing mandatory 'Maintainer' field".to_string())?;

        // Optional fields parsing
        let description = fields.remove("Description");
        let section = fields.remove("Section");
        let priority = fields.remove("Priority");
        let homepage = fields.remove("Homepage");
        let built_using = fields.remove("Built-Using");
        let original_maintainer = fields.remove("Original-Maintainer");

        // Parse dependency-like fields using the new helper functions
        let depends = fields.remove("Depends").map_or_else(Vec::new, |s| Self::parse_dependencies(&s));
        let recommends = fields.remove("Recommends").map_or_else(Vec::new, |s| Self::parse_dependencies(&s));
        let suggests = fields.remove("Suggests").map_or_else(Vec::new, |s| Self::parse_dependencies(&s));
        let conflicts = fields.remove("Conflicts").map_or_else(Vec::new, |s| Self::parse_dependencies(&s).into_iter().flatten().collect());
        let replaces = fields.remove("Replaces").map_or_else(Vec::new, |s| Self::parse_dependencies(&s).into_iter().flatten().collect());
        let provides = fields.remove("Provides").map_or_else(Vec::new, |s| Self::parse_provides(&s));


        Ok(Package {
            package,
            version,
            maintainer,
            description,
            suite,
            component,
            architecture,
            section,
            priority,
            homepage,
            depends,
            recommends,
            suggests,
            conflicts,
            replaces,
            provides,
            built_using,
            original_maintainer,
            other_fields: fields, // Remaining fields
        })
    }
}


impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Package: {}\nVersion: {}\nSuite: {}\nComponent: {}\nArchitecture: {}\nMaintainer: {}",
            self.package.red().bold(),
            self.version.green(),
            self.suite.yellow(),
            self.component.magenta(),
            self.architecture.as_ref().map_or("N/A".to_string(), |s| s.blue().to_string()),
            self.maintainer.cyan(),
        )?;

        if let Some(description) = &self.description {
            writeln!(f, "Description: {}", description.truecolor(255, 165, 0))?; // Orange for description
        }

        if let Some(section) = &self.section {
            writeln!(f, "Section: {}", section.blue())?;
        }
        if let Some(priority) = &self.priority {
            writeln!(f, "Priority: {}", priority.green())?;
        }
        if let Some(homepage) = &self.homepage {
            writeln!(f, "Homepage: {}", homepage.blue().underline())?;
        }

        if !self.depends.is_empty() {
            writeln!(f, "Depends:")?;
            for group in &self.depends {
                let alts: Vec<String> = group.iter().map(|pr| format!("{}", pr)).collect();
                writeln!(f, "  - {}", alts.join(" | ").purple())?;
            }
        }
        if !self.recommends.is_empty() {
            writeln!(f, "Recommends:")?;
            for group in &self.recommends {
                let alts: Vec<String> = group.iter().map(|pr| format!("{}", pr)).collect();
                writeln!(f, "  - {}", alts.join(" | ").bright_purple())?;
            }
        }
        if !self.suggests.is_empty() {
            writeln!(f, "Suggests:")?;
            for group in &self.suggests {
                let alts: Vec<String> = group.iter().map(|pr| format!("{}", pr)).collect();
                writeln!(f, "  - {}", alts.join(" | ").yellow())?;
            }
        }
        if !self.conflicts.is_empty() {
            writeln!(f, "Conflicts:")?;
            for conflict in &self.conflicts {
                writeln!(f, "  - {}", format!("{}", conflict).red())?;
            }
        }
        if !self.replaces.is_empty() {
            writeln!(f, "Replaces:")?;
            for replace in &self.replaces {
                writeln!(f, "  - {}", format!("{}", replace).red())?;
            }
        }
        if !self.provides.is_empty() {
            writeln!(f, "Provides:")?;
            for provide in &self.provides {
                writeln!(f, "  - {}", format!("{}", provide).cyan())?;
            }
        }
        
        if let Some(built_using) = &self.built_using {
            writeln!(f, "Built-Using: {}", built_using.yellow())?;
        }
        if let Some(original_maintainer) = &self.original_maintainer {
            writeln!(f, "Original-Maintainer: {}", original_maintainer.cyan())?;
        }

        // Display any other fields
        for (key, value) in &self.other_fields {
            writeln!(f, "{}: {}", key.white().italic(), value.white())?;
        }

        Ok(())
    }
}

// Package から PackageData への変換
impl From<Package> for PackageData {
    fn from(pkg: Package) -> Self {
        let (author_name, author_email) = parse_maintainer(&pkg.maintainer);

        let mut relation_data = RelationData {
            depend: pkg.depends,
            recommends: pkg.recommends,
            suggests: pkg.suggests,
            conflicts: pkg.conflicts,
            virtuals: pkg.provides, // DebianのProvidesはipakのvirtualsに相当
            ..Default::default()
        };

        // Replaces フィールドはipakのRelationDataに直接のマッピングがないため、
        // conflicts に追加する（意味合いが近いため）か、other_fields に残すか、今回は conflicts に追加します。
        // ただし、厳密には Replaces と Conflicts は異なるため、注意が必要です。
        // 今回はipak::modules::pkg::PackageDataの既存のフィールドに合わせるため、
        // ここでは簡単にconflictsに含めています。
        // もしipak::modules::pkg::PackageDataの定義を変更できるなら、Replaces用のフィールドを追加するのが理想的です。
        relation_data.conflicts.extend(pkg.replaces);


        let package_data = PackageData {
            about: AboutData {
                author: AuthorAboutData {
                    name: author_name,
                    email: author_email,
                },
                package: PackageAboutData {
                    name: pkg.package,
                    version: Version::from_str(&pkg.version).unwrap_or_default(), // Debianのバージョン文字列をipak::Versionに変換
                },
            },
            architecture: pkg.architecture.map(|arch| vec![arch]).unwrap_or_default(),
            mode: ipak::modules::pkg::Mode::Any, // Debian Packagesファイルはインストールモードを直接示さないためAnyとする
            relation: relation_data,
        };

        // description、section、priority、homepage、built_using、original_maintainer
        // そして other_fields の残りを PackageData の other_fields に追加
        // ただし、PackageData には other_fields が存在しないため、直接追加することはできません。
        // PackageData の構造に合わせて、直接マッピングできないフィールドは捨てるか、
        // PackageData 側に拡張フィールドを設ける必要があります。
        // ここでは、PackageData の定義に合わせて、これらを追加で保持する場所がないため、
        // 単純に変換時に破棄されることになります。
        // もし保持したい場合は、PackageData 構造体の定義を変更する必要があります。
        
        // 例: Debug用に、変換時に捨てられるフィールドを一時的に表示
        #[cfg(debug_assertions)]
        {
            if let Some(desc) = pkg.description {
                eprintln!("Debug: Description '{}' is not directly mapped to PackageData.", desc);
            }
            if let Some(section) = pkg.section {
                eprintln!("Debug: Section '{}' is not directly mapped to PackageData.", section);
            }
            // 他のフィールドも同様
        }

        package_data
    }
}

// Maintainer フィールドを名前とメールアドレスにパースするヘルパー関数
fn parse_maintainer(maintainer_str: &str) -> (String, String) {
    if let Some(start_paren) = maintainer_str.rfind('<') {
        if let Some(end_paren) = maintainer_str.rfind('>') {
            if end_paren > start_paren {
                let name = maintainer_str[..start_paren].trim().to_string();
                let email = maintainer_str[start_paren + 1..end_paren].trim().to_string();
                return (name, email);
            }
        }
    }
    // デフォルト値またはパースできない場合
    (maintainer_str.to_string(), String::new())
}


async fn fetch_url(client: &Client, url: &str) -> Result<String, io::Error> {
    let response = client.get(url).send().await
        .map_err(|e| io::Error::new(io::ErrorKind::ConnectionRefused, format!("Failed to send request to {}: {}", url, e)))?;

    if !response.status().is_success() {
        return Err(io::Error::other(format!("HTTP error: {} for URL: {}", response.status(), url)));
    }
    let bytes = response.bytes().await
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read response bytes: {}", e)))?;
    
    if url.ends_with(".gz") {
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut content = String::new();
        decoder.read_to_string(&mut content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to decompress gzip content: {}", e)))?;
        Ok(content)
    } else {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to decode UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipak::modules::pkg::Mode; // ipak の Mode を使用

    #[tokio::test]
    async fn test_package_to_packagedata_conversion() {
        // テスト用の Debian Package を作成
        let debian_pkg = Package {
            package: "example-package".to_string(),
            version: "1.2.3-1ubuntu1".to_string(),
            maintainer: "John Doe <john.doe@example.com>".to_string(),
            description: Some("A sample package for testing conversion.".to_string()),
            suite: "noble".to_string(),
            component: "main".to_string(),
            architecture: Some("amd64".to_string()),
            section: Some("misc".to_string()),
            priority: Some("optional".to_string()),
            homepage: Some("http://example.com".to_string()),
            depends: Package::parse_dependencies("dep-a (>= 1.0) | dep-b (= 2.0), dep-c"),
            recommends: Package::parse_dependencies("rec-x"),
            suggests: Package::parse_dependencies("sugg-y (>> 3.0)"),
            conflicts: Package::parse_dependencies("conflict-old (<< 1.0)").into_iter().flatten().collect(),
            replaces: Package::parse_dependencies("old-replaced-pkg").into_iter().flatten().collect(),
            provides: Package::parse_provides("virtual-pkg (= 4.0), another-virtual"),
            built_using: None,
            original_maintainer: None,
            other_fields: HashMap::new(),
        };

        // Package から PackageData へ変換
        let ipak_pkg_data: PackageData = debian_pkg.into();

        // 変換結果の検証
        assert_eq!(ipak_pkg_data.about.package.name, "example-package");
        assert_eq!(ipak_pkg_data.about.package.version.to_string(), "1.2.3-1ubuntu1"); // Versionの文字列比較
        assert_eq!(ipak_pkg_data.about.author.name, "John Doe");
        assert_eq!(ipak_pkg_data.about.author.email, "john.doe@example.com");
        assert_eq!(ipak_pkg_data.architecture, vec!["amd64".to_string()]);
        assert_eq!(ipak_pkg_data.mode, Mode::Any); // デフォルトでAnyになることを確認

        // 依存関係の検証 (例としてDependsの一部を検証)
        assert!(!ipak_pkg_data.relation.depend.is_empty());
        assert_eq!(ipak_pkg_data.relation.depend[0][0].name, "dep-a");
        assert_eq!(ipak_pkg_data.relation.depend[0][0].range.to_string(), ">= 1.0");
        assert_eq!(ipak_pkg_data.relation.depend[0][1].name, "dep-b");
        assert_eq!(ipak_pkg_data.relation.depend[0][1].range.to_string(), "= 2.0");
        assert_eq!(ipak_pkg_data.relation.depend[1][1].name, "dep-c");
        assert_eq!(ipak_pkg_data.relation.depend[1][1].range.to_string(), ""); // dep-c にはバージョン範囲がない

        // Conflicts と Replaces の統合の検証
        assert!(!ipak_pkg_data.relation.conflicts.is_empty());
        // conflict-oldとold-replaced-pkgの両方が含まれていることを確認
        let conflict_names: Vec<String> = ipak_pkg_data.relation.conflicts.iter().map(|pr| pr.name.clone()).collect();
        assert!(conflict_names.contains(&"conflict-old".to_string()));
        assert!(conflict_names.contains(&"old-replaced-pkg".to_string()));

        // Provides (virtuals) の検証
        assert!(!ipak_pkg_data.relation.virtuals.is_empty());
        assert_eq!(ipak_pkg_data.relation.virtuals[0].name, "virtual-pkg");
        assert_eq!(ipak_pkg_data.relation.virtuals[0].version.to_string(), "4.0");


        println!("\n--- Converted PackageData ---");
        println!("{}", ipak_pkg_data); // PackageData の Display を利用して出力
    }
}