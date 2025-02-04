use std::error::Error;
use std::process;

use tokio;
use figlet_rs::FIGfont;

use rs_manifest_patcher::prompt;
use rs_manifest_patcher::{Config, Manifest, Transaction};


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
    let slant_font_data = include_str!("../resources/slant.flf");
    let slant_font = FIGfont::from_content(slant_font_data).unwrap();
    let figure = slant_font.convert("Banner");
    print!("{}", figure.unwrap());
    println!("Bugs or issues: https://github.com/sogladev/rs_manifest_patcher/");

    let manifest = Manifest::build(&config.manifest_location).await?;
    let transaction = Transaction::new(&manifest);

    transaction.print();
    if !prompt::confirm("Is this ok")? {
        process::exit(1);
    }

    if transaction.has_pending_operations() {
        transaction.download().await?;
    }

    println!("\n{}", "-".repeat(96));
    println!("All files are up to date or successfully downloaded.");

    Ok(())
}