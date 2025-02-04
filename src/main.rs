use std::fs;
use std::process;
use std::error::Error;

use rs_manifest_patcher::config::Config;
use rs_manifest_patcher::Location;

fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = match config.manifest_location {
        Location::Url(url) => {
            println!("URL: {url}");
            let mut response = Vec::new();
            http_req::request::get(url.as_str(), &mut response)
                .map_err(|err| format!("HTTP request failed: {}", err))?;
            let contents = String::from_utf8(response)
                .map_err(|err| format!("Parsing response as UTF-8 failed: {}", err))?;
            "Not implemented yet".to_string()
        }
        Location::FilePath(file_path) => {
            println!("File path: {:?}", file_path);
            fs::read_to_string(file_path)?
        }
    };

    println!("With text:\n{contents}");

    Ok(())
}