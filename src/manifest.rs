use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use url::Url;

#[derive(Debug, Clone)]
pub enum Location {
    Url(Url),
    FilePath(PathBuf),
}

impl Location {
    /// Parse a manifest location string into a `Location` enum
    pub fn parse(manifest_str: String) -> Result<Self, &'static str> {
        if let Ok(parsed_url) = Url::parse(&manifest_str) {
            // A relative URL string should return an error
            if parsed_url.cannot_be_a_base() {
                return Err("URL is incomplete");
            }
            // Check if the URL has a valid scheme
            if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                return Ok(Location::Url(parsed_url));
            }
        }

        let path = PathBuf::from(&manifest_str);
        if path.exists() && fs::File::open(&path).is_ok() {
            return Ok(Location::FilePath(path));
        }

        Err(
            "Manifest location must be a valid URL (e.g., http://localhost:8080/manifest.json) \
            or a readable file path"
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PatchFile {
    pub path: String,
    pub hash: String,
    pub size: i64,
    pub custom: bool,
    #[serde(rename = "URL")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub version: String,
    pub files: Vec<PatchFile>,
}

impl Manifest {
    fn from_file(file_path: &PathBuf) -> Result<String, std::io::Error> {
        fs::read_to_string(file_path)
    }

    async fn from_url(url: &Url) -> Result<String, Box<dyn Error>> {
        let response = reqwest::get(url.as_str()).await?;
        let contents = response.text().await?;
        Ok(contents)
    }

    /// Retrieve Manifest Data
    pub async fn build(location: &Location) -> Result<Manifest, Box<dyn Error>> {
        let contents = match location {
            Location::Url(url) => Manifest::from_url(&url).await?,
            Location::FilePath(file_path) => Manifest::from_file(&file_path)?
        };

        let manifest: Manifest = serde_json::from_str(&contents)?;
        Ok(manifest)
    }
}
