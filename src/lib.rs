
use clap::{arg, Command};
use std::net::IpAddr;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
pub enum ManifestLocation {
    IpAddr(IpAddr),
    FilePath(PathBuf),
}

#[derive(Debug)]
pub struct Config {
    pub manifest: ManifestLocation,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let matches = Command::new("rs_manifest_patcher")
            // .version("1.0")
            // .about("Does awesome things")
            .arg(arg!(-m --manifest <String> "Sets a custom manifest")
                    .default_value("manifest.json"))
            .get_matches();

        let manifest_str = matches.get_one::<String>("manifest").unwrap().to_string();

        // Validate if manifest is either an IP address or a readable file path
        let manifest = if let Ok(ip_addr) = manifest_str.parse::<IpAddr>() {
            ManifestLocation::IpAddr(ip_addr)
        } else {
            let path = PathBuf::from(&manifest_str);
            if path.try_exists().is_ok() && fs::File::open(&path).is_ok() {
                ManifestLocation::FilePath(path)
            } else {
                return Err("Manifest must be a valid IP address or a readable file path");
            }
        };

        dbg!(&manifest);
        Ok(Config { manifest })
    }
}