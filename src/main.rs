use std::error::Error;
use std::process;

use rs_manifest_patcher::{banner, prompt};
use rs_manifest_patcher::{Config, Manifest, Transaction};

#[tokio::main]
async fn main() {
    #[cfg(not(unix))]
    colored::control::set_virtual_terminal(true).unwrap();

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
    banner::print_banner();

    let base_path = std::env::current_dir().expect("Failed to get current directory");
    let manifest = Manifest::build(&config.manifest_location).await?;
    let transaction = Transaction::new(manifest, base_path);

    transaction.print();
    if !prompt::confirm("Is this ok")? {
        process::exit(1);
    }

    if transaction.has_pending_operations() {
        transaction.download().await?;
    }

    println!("\n{}", "-".repeat(100));
    println!("All files are up to date or successfully downloaded.");

    Ok(())
}
