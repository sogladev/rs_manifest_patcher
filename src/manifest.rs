use std::fs;
use std::path::PathBuf;
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
        if path.try_exists().is_ok() && fs::File::open(&path).is_ok() {
            return Ok(Location::FilePath(path));
        }

        Err(
            "Manifest location must be a valid URL (e.g., http://localhost:8080/manifest.json) \
            or a readable file path"
        )
    }
}
