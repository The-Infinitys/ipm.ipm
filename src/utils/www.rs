use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Represents a parsed URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct URL {
    protocol: String,
    domain: String, // Changed to String for simpler domain representation
    path: PathBuf,
}

impl URL {
    /// Creates a new URL instance.
    pub fn new(
        protocol: impl Into<String>,
        domain: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        URL {
            protocol: protocol.into(),
            domain: domain.into(),
            path: PathBuf::from("/").join(path),
        }
    }
    pub fn join(
        self,
        path: &str,
    ) -> Result<Self, std::io::Error> {
        Ok(Self::new(
            self.protocol,
            self.domain,
            self.path.join(path),
        ))
    }
    /// Fetches binary data from the URL.
    /// Returns the response body as a Vec<u8> or a boxed error.
    pub fn fetch_bin(
        &self,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let request_url = self.to_string();
        let response = reqwest::blocking::get(&request_url)?;
        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| e.into())
    }
    /// Fetches data from the URL.
    /// Returns the response body as a String or a boxed error.
    pub fn fetch(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request_url = self.to_string();
        let response = reqwest::blocking::get(&request_url)?; // Use ? for error propagation

        response.text().map_err(|e| e.into()) // Convert reqwest::Error to Box<dyn std::error::Error>
    }

    /// Parses a URL string into a URL instance.
    pub fn parse(url_str: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = url_str.split("://").collect();
        if parts.len() != 2 {
            return Err(
                "Invalid URL format: missing protocol or malformed",
            );
        }

        let protocol = parts[0].to_string();
        let rest = parts[1];

        let mut domain_path_parts = rest.splitn(2, '/'); // Split only on the first '/'
        let domain = domain_path_parts
            .next()
            .ok_or("Invalid URL format: missing domain")?
            .to_string();

        let path =
            if let Some(path_str) = domain_path_parts.next() {
                PathBuf::from(path_str)
            } else {
                PathBuf::new()
            };

        Ok(Self::new(protocol, domain, path))
    }

    /// Returns the protocol of the URL.
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Returns the domain of the URL.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the path of the URL.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

// Implements `FromStr` trait for easy conversion from string slices to `URL`.
impl FromStr for URL {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        URL::parse(s)
    }
}

// Implements `Display` trait for easily formatting `URL` instances into strings.
impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.as_os_str().is_empty() {
            write!(f, "{}://{}", self.protocol, self.domain)
        } else {
            write!(
                f,
                "{}://{}{}",
                self.protocol,
                self.domain,
                self.path.display()
            )
        }
    }
}

/// Trait for converting types to a `URL` instance.
pub trait ToURL {
    fn to_url(&self) -> Result<URL, &'static str>;
}

// Implement `ToURL` for `String`.
impl ToURL for String {
    fn to_url(&self) -> Result<URL, &'static str> {
        URL::parse(self)
    }
}

// Implement `ToURL` for `&str`.
impl ToURL for &str {
    fn to_url(&self) -> Result<URL, &'static str> {
        URL::parse(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_url_parse() {
        let url = URL::parse(
            "https://www.example.com/path/to/resource",
        )
        .unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.domain, "www.example.com");
        assert_eq!(url.path, PathBuf::from("/path/to/resource"));

        let url_no_path =
            URL::parse("http://example.org").unwrap();
        assert_eq!(url_no_path.protocol, "http");
        assert_eq!(url_no_path.domain, "example.org");
        assert_eq!(url_no_path.path, PathBuf::from("/"));
    }

    #[test]
    fn test_url_to_string() {
        let url = URL::new(
            "https",
            "www.example.com",
            PathBuf::from("path/to/resource"),
        );
        assert_eq!(
            url.to_string(),
            "https://www.example.com/path/to/resource"
        );

        let url_no_path =
            URL::new("http", "example.org", PathBuf::new());
        assert_eq!(
            url_no_path.to_string(),
            "http://example.org/"
        );
    }

    #[test]
    fn test_from_str() {
        let url: URL =
            "https://www.example.com/path/to/resource"
                .parse()
                .unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.domain, "www.example.com");
        assert_eq!(url.path, PathBuf::from("/path/to/resource"));
    }

    #[test]
    fn test_display() {
        let url = URL::new(
            "https",
            "www.example.com",
            PathBuf::from("path/to/resource"),
        );
        assert_eq!(
            format!("{}", url),
            "https://www.example.com/path/to/resource"
        );
    }

    #[test]
    fn test_to_url_trait() {
        let url_str = "https://www.example.com/path";
        let url = url_str.to_url().unwrap();
        assert_eq!(url.protocol(), "https");
        assert_eq!(url.domain(), "www.example.com");

        let url_string =
            String::from("http://localhost:8080/api");
        let url_from_string = url_string.to_url().unwrap();
        assert_eq!(url_from_string.protocol(), "http");
        assert_eq!(url_from_string.domain(), "localhost:8080");
        assert_eq!(
            url_from_string.path(),
            &PathBuf::from("/api")
        );
    }

    #[test]
    fn test_serde() {
        let url = URL::new(
            "https",
            "www.example.com",
            PathBuf::from("path"),
        );

        // Serialize
        let serialized = serde_json::to_string(&url).unwrap();

        // Deserialize
        let deserialized: URL =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(url.to_string(), deserialized.to_string());
    }

    #[test]
    fn test_getters() {
        let url = URL::new(
            "https",
            "www.example.com",
            PathBuf::from("path"),
        );

        assert_eq!(url.protocol(), "https");
        assert_eq!(url.domain(), "www.example.com");
        assert_eq!(url.path(), &PathBuf::from("/path"));
    }
}
