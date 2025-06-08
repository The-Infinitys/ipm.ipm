use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::path::PathBuf;
use std::str::{Bytes, FromStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct URL {
    protocol: String,
    domain: Vec<String>,
    path: PathBuf,
}

impl URL {
    /// 新しいURLインスタンスを作成します
    pub fn new(
        protocol: impl Into<String>,
        domain: Vec<String>,
        path: PathBuf,
    ) -> Self {
        URL { protocol: protocol.into(), domain, path }
    }
    pub fn fetch(&self) -> Result<Bytes, i32> {
        Ok(())
    }
    pub fn fetch_str(&self) -> Result<String, i32> {
        match self.fetch() {
                Ok(byte_data) => Ok(byte_data.to_string()),
                Err(e) => Err(e),
            
        })
    }
    /// URL文字列からURLインスタンスを作成します
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format");
        }

        let protocol = parts[0].to_string();
        let rest = parts[1];

        let domain_path: Vec<&str> = rest.split('/').collect();
        let domain = domain_path[0]
            .split('.')
            .map(|s| s.to_string())
            .collect();

        let path = if domain_path.len() > 1 {
            PathBuf::from(domain_path[1..].join("/"))
        } else {
            PathBuf::new()
        };

        Ok(URL { protocol, domain, path })
    }

    /// URLの文字列表現を取得します
    pub fn to_string(&self) -> String {
        format!(
            "{}://{}/{}",
            self.protocol,
            self.domain.join("."),
            self.path.display()
        )
    }

    /// URLからデータを取得するメソッド群
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub fn domain(&self) -> &[String] {
        &self.domain
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn domain_string(&self) -> String {
        self.domain.join(".")
    }
}

impl FromStr for URL {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        URL::parse(s)
    }
}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}://{}/{}",
            self.protocol,
            self.domain.join("."),
            self.path.display()
        )
    }
}

// StrToURLトレイトの定義
pub trait StrToURL {
    fn to_url(&self) -> Result<URL, &'static str>;
}

// String用の実装
impl StrToURL for String {
    fn to_url(&self) -> Result<URL, &'static str> {
        URL::parse(self)
    }
}

// &str用の実装
impl StrToURL for &str {
    fn to_url(&self) -> Result<URL, &'static str> {
        URL::parse(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parse() {
        let url = URL::parse(
            "https://www.example.com/path/to/resource",
        )
        .unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.domain, vec!["www", "example", "com"]);
        assert_eq!(url.path, PathBuf::from("path/to/resource"));
    }

    #[test]
    fn test_url_to_string() {
        let url = URL::new(
            "https",
            vec![
                "www".to_string(),
                "example".to_string(),
                "com".to_string(),
            ],
            PathBuf::from("path/to/resource"),
        );
        assert_eq!(
            url.to_string(),
            "https://www.example.com/path/to/resource"
        );
    }

    #[test]
    fn test_from_str() {
        let url = "https://www.example.com/path/to/resource"
            .parse::<URL>()
            .unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.domain, vec!["www", "example", "com"]);
        assert_eq!(url.path, PathBuf::from("path/to/resource"));
    }

    #[test]
    fn test_display() {
        let url = URL::new(
            "https",
            vec![
                "www".to_string(),
                "example".to_string(),
                "com".to_string(),
            ],
            PathBuf::from("path/to/resource"),
        );
        assert_eq!(
            format!("{}", url),
            "https://www.example.com/path/to/resource"
        );
    }

    #[test]
    fn test_str_to_url() {
        let url_str = "https://www.example.com/path";
        let url = url_str.to_url().unwrap();
        assert_eq!(url.protocol(), "https");
        assert_eq!(url.domain_string(), "www.example.com");
    }

    #[test]
    fn test_serde() {
        let url = URL::new(
            "https",
            vec![
                "www".to_string(),
                "example".to_string(),
                "com".to_string(),
            ],
            PathBuf::from("path"),
        );

        // シリアライズ
        let serialized = serde_json::to_string(&url).unwrap();

        // デシリアライズ
        let deserialized: URL =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(url.to_string(), deserialized.to_string());
    }

    #[test]
    fn test_getters() {
        let url = URL::new(
            "https",
            vec![
                "www".to_string(),
                "example".to_string(),
                "com".to_string(),
            ],
            PathBuf::from("path"),
        );

        assert_eq!(url.protocol(), "https");
        assert_eq!(url.domain_string(), "www.example.com");
        assert_eq!(url.path(), &PathBuf::from("path"));
    }
}
