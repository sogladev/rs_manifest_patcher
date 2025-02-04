use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Location {
    Url(IpAddr),
    FilePath(PathBuf),
}

impl Location {
    /// Attempt to parse the manifest string as either an IP address or a readable file path
    pub fn parse(manifest_str: String) -> Result<Self, &'static str> {
        let manifest = if let Ok(url) = manifest_str.parse::<IpAddr>() {
            Ok(Location::Url(url))
        } else {
            let path = PathBuf::from(&manifest_str);
            if path.try_exists().is_ok() && fs::File::open(&path).is_ok() {
                Ok(Location::FilePath(path))
            } else {
                Err(
                    "Manifest location must be a valid URL (e.g., http://localhost:8080/manifest.json) \
                    or a readable file path"
                )
            }
        };
        manifest
    }
}