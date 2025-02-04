
use clap::{arg, Command};

#[derive(Debug)]
pub struct Config {
    pub manifest: String,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let matches = Command::new("rs_manifest_patcher")
            // .version("1.0")
            .about("Does awesome things")
            .arg(arg!(-m --manifest <String> "Sets a custom manifest")
                    .default_value("manifest.json"))
            .get_matches();

        let manifest =matches.get_one::<String>("manifest").unwrap().to_string();
        dbg!(&manifest);
        Ok(Config { manifest })
    }
}