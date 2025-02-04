use clap::{arg, Command};

use crate::manifest;

#[derive(Debug)]
pub struct Config {
    pub manifest_location: manifest::Location,
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

        let manifest = manifest::Location::parse(manifest_str)?;

        dbg!(&manifest);
        Ok(Config { manifest_location: manifest })
    }
}