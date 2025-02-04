
pub struct Config {
    pub manifest: String,
}

impl Config {
    pub fn build( mut args: impl Iterator<Item = String>,) -> Result<Config, &'static str> {
        args.next(); // Skip program name

        // let manifest = match args.next() {
            // Some(arg) => arg,
            // None => return Err("Didn't get a manifest string"),
        // };

        let manifest: String = args.next().unwrap_or_else(|| "manifest.json".to_string());

        // let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            manifest,
        })
    }
}