use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use colored::*;
use deb822_lossless::Deb822;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::io::Read;
use std::str::FromStr;

// opak::modules::pkgからのインポートを想定
// 実際にはクレートの構造に合わせてパスを調整してください
use ipak::modules::pkg::{PackageRange, PackageVersion};
use ipak::modules::version::{Version, VersionRange};


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

/// Represents a single package entry as defined in Debian Policy Manual.
/// Common fields are parsed into their own fields for easier access and type safety.
/// All other fields are stored in a HashMap.
pub struct Package {
    // Mandatory fields
    pub package: String, // 'Package' field
    pub version: String, // 'Version' field (Debian policy version string, not ipak::modules::pkg::Version)
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
            .filter_map(|group_str| {
                let alternatives: Vec<PackageRange> = group_str.split('|')
                    .filter_map(|dep_part| {
                        let trimmed_dep = dep_part.trim();
                        // Example: "pkg-name (>= 1.0)" or "pkg-name"
                        let parts: Vec<&str> = trimmed_dep.splitn(2, ' ').collect();
                        let name = parts[0].to_string();
                        let range = if parts.len() > 1 {
                            // Version range exists, e.g., "(>= 1.0)"
                            let version_range_str = parts[1].trim_matches(|c| c == '(' || c == ')');
                            VersionRange::from_str(version_range_str).ok()?
                        } else {
                            // No version range specified, assume any version
                            VersionRange::from_str("").ok()? // Represents any version
                        };
                        Some(PackageRange { name, range })
                    })
                    .collect();
                if alternatives.is_empty() {
                    None
                } else {
                    Some(alternatives)
                }
            })
            .collect()
    }

    /// Parses a single Provides string (e.g., "virtual-pkg (= 1.0), another-virtual")
    /// into a vector of PackageVersion.
    fn parse_provides(provides_str: &str) -> Vec<PackageVersion> {
        provides_str.split(',')
            .filter_map(|provide_part| {
                let trimmed_provide = provide_part.trim();
                let parts: Vec<&str> = trimmed_provide.splitn(2, ' ').collect();
                let name = parts[0].to_string();
                let version = if parts.len() > 1 {
                    let version_str = parts[1].trim_matches(|c| c == '(' || c == ')');
                    Version::from_str(version_str).ok()?
                } else {
                    Version::default() // No version specified, use default
                };
                Some(PackageVersion { name, version })
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

pub async fn fetch_package_indices(apt_info: &AptInfo) -> Result<Vec<Package>, Box<dyn Error>> {
    let client = Client::new();
    let mut packages = Vec::new();
    let uri = &apt_info.uris;

    match apt_info.types {
        AptType::Deb => {
            for suite in &apt_info.suites {
                for component in &apt_info.components {
                    for arch in &apt_info.architectures {
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
                                Err(e) => eprintln!("Warning: Skipping package due to parsing error: {}", e),
                            }
                        }
                    }
                }
            }
        }
        AptType::DebSrc => {
            for suite in &apt_info.suites {
                for component in &apt_info.components {
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

async fn fetch_url(client: &Client, url: &str) -> Result<String, Box<dyn Error>> {
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {} for URL: {}", response.status(), url).into());
    }
    let bytes = response.bytes().await?;
    
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

    #[tokio::test]
    async fn test_fetch_noble_source_packages() {
        let apt_info = AptInfo {
            types: AptType::DebSrc,
            uris: "http://archive.ubuntu.com/ubuntu".to_string(),
            suites: vec!["noble".to_string()],
            components: vec!["main".to_string()],
            architectures: Vec::new(), // Not applicable for deb-src
        };

        println!("Repository Info:\n{}", apt_info);

        match fetch_package_indices(&apt_info).await {
            Ok(packages) => {
                for package in packages.iter().take(5) { // 最初の5ソースパッケージを表示
                    println!("\n{}", package);
                }
                println!("\nTotal source packages: {}", packages.len());
                assert!(!packages.is_empty(), "No source packages were fetched");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("Failed to fetch source packages: {}", e);
            }
        }
    }
}