use clap::{arg, Command};
use std::net::IpAddr;
use std::path::PathBuf;
use std::fs;

use crate::manifest;

#[derive(Debug)]
pub struct Config {
    pub manifest: manifest::Location,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let matches = Command::new("rs_manifest_patcher")
            // .version("1.0")
            // .about("Does awesome things")
            .arg(arg!(-m --manifest <String> "Path to manifest.json file or URL (e.g., http://localhost:8080/manifest.json)")
                .default_value("manifest.json"))
            .get_matches();

        let manifest_str = matches.get_one::<String>("manifest").unwrap().to_string();

        // Validate if manifest is either an IP address or a readable file path
        let manifest = if let Ok(url) = manifest_str.parse::<IpAddr>() {
            manifest::Location::Url(url)
        } else {
            let path = PathBuf::from(&manifest_str);
            if path.try_exists().is_ok() && fs::File::open(&path).is_ok() {
                manifest::Location::FilePath(path)
            } else {
                return Err("Manifest location must be a valid URL (e.g., http://localhost:8080/manifest.json) or a readable file path");
            }
        };

        dbg!(&manifest);
        Ok(Config { manifest })
    }
}