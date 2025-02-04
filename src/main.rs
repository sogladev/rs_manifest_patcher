use std::fs;
use std::process;
use std::error::Error;

use rs_manifest_patcher::Config;

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
    let contents = fs::read_to_string(config.manifest)?;

    println!("With text:\n{contents}");

    Ok(())
}