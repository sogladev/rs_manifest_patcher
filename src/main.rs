use std::process;
use std::error::Error;

use rs_manifest_patcher::manifest;
use rs_manifest_patcher::manifest::FileOperation;
use rs_manifest_patcher::prompt;
use tokio;

use rs_manifest_patcher::Config;
use rs_manifest_patcher::Manifest;
use std::path::Path;
use reqwest;

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
    if !prompt::confirm("Is this ok")? {
        process::exit(1);
    }
    // Download Files, create directories if needed
    for op in file_operations {
        let dest_path = Path::new(&op.patch_file.path);
        if let Some(dir) = dest_path.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }

        let response = reqwest::get(&op.patch_file.url).await?;
        if !response.status().is_success() {
            eprintln!("Failed to download {}: {}", &op.patch_file.url, response.status());
            continue;
        }

        let content = response.bytes().await?;
        tokio::fs::write(dest_path, &content).await?;
        println!("Downloaded {} to {:?}", &op.patch_file.url, dest_path);
    }
    Ok(())
}