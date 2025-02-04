
use clap::{arg, Command};
use std::net::IpAddr;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub manifest: String,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let matches = Command::new("rs_manifest_patcher")
            // .version("1.0")
            // .about("Does awesome things")
            .arg(arg!(-m --manifest <String> "Sets a custom manifest")
                    .default_value("manifest.json"))
            .get_matches();

        let manifest = matches.get_one::<String>("manifest").unwrap().to_string();

        // Validate if manifest is either an IP address or a file path
        if  Path::new(&manifest).try_exists().is_err() || manifest.parse::<IpAddr>().is_err() {
            return Err("Manifest must be a valid IP address or an existing file path");
        }

        dbg!(&manifest);
        Ok(Config { manifest })
    }
}