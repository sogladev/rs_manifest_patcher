use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
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
            or a readable file path",
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
    /// Create a new Manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let mut manifest: Manifest = serde_json::from_str(json)?;

        // Convert paths from Windows to Unix format
        manifest
            .files
            .iter_mut()
            .for_each(|file| file.path = file.path.replace("\\", "/"));

        Ok(manifest)
    }

    /// Load manifest from a file
    pub fn from_file(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(file_path)?;
        Self::from_json(&contents)
    }

    /// Build manifest from a location (URL or file)
    pub async fn build(location: &Location) -> Result<Self, Box<dyn Error>> {
        match location {
            Location::Url(url) => {
                let response = reqwest::get(url.as_str()).await?;
                let contents = response.text().await?;
                Self::from_json(&contents)
            }
            Location::FilePath(file_path) => Self::from_file(file_path),
        }
    }
}
