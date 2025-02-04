use std::fs;
use std::process;
use std::error::Error;

use rs_manifest_patcher::Config;
use rs_manifest_patcher::ManifestLocation;

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
    let contents = match config.manifest {
        ManifestLocation::IpAddr(ip_addr) => {
            println!("IP address: {ip_addr}");
            "Not implemented yet".to_string()
        }
        ManifestLocation::FilePath(file_path) => {
            println!("File path: {:?}", file_path);
            fs::read_to_string(file_path)?
        }
    };

    println!("With text:\n{contents}");

    Ok(())
}