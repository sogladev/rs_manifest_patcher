use std::process;
use std::error::Error;

use rs_manifest_patcher::manifest;
use rs_manifest_patcher::manifest::FileOperation;
use tokio;

use rs_manifest_patcher::Config;
use rs_manifest_patcher::Manifest;

#[tokio::main]
async fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config).await {
        println!("Application error: {e}");
        process::exit(1);
    }
}

async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let manifest = Manifest::build(&config.manifest_location).await?;
    dbg!(&manifest);
    let file_operations = FileOperation::process(&manifest);
    dbg!(&file_operations);
    manifest::show_transaction_overview(&manifest, &file_operations);
    Ok(())
}