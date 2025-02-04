use std::{fs, os::unix::fs::MetadataExt};
use std::path::PathBuf;
use std::error::Error;

use url::Url;
use serde::{Serialize, Deserialize};
use colored::Colorize;
use humansize;
use humansize::BINARY;

#[derive(Debug, Clone)]
pub enum Location {
    Url(Url),
    FilePath(PathBuf),
}

impl Location {
    /// Parse a manifest location string into a `Location` enum
    pub fn parse(manifest_str: String) -> Result<Self, &'static str> {
        if let Ok(parsed_url) = Url::parse(&manifest_str) {
            // A relative URL string should return an error
            if parsed_url.cannot_be_a_base() {
                return Err("URL is incomplete");
            }
            // Check if the URL has a valid scheme
            if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                return Ok(Location::Url(parsed_url));
            }
        }

        let path = PathBuf::from(&manifest_str);
        if path.exists() && fs::File::open(&path).is_ok() {
            return Ok(Location::FilePath(path));
        }

        Err(
            "Manifest location must be a valid URL (e.g., http://localhost:8080/manifest.json) \
            or a readable file path"
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PatchFile {
    pub path: String,
    pub hash: String,
    pub size: i64,
    pub custom: bool,
    #[serde(rename = "URL")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub version: String,
    pub files: Vec<PatchFile>,
}

impl Manifest {
    fn from_file(file_path: &PathBuf) -> Result<String, std::io::Error> {
        fs::read_to_string(file_path)
        // fs::read_to_string(file_path).unwrap().expect("Failed to read file")
    }

    async fn from_url(url: &Url) -> Result<String, Box<dyn Error>> {
        let response = reqwest::get(url.as_str()).await?;
        let contents = response.text().await?;
        Ok(contents)
    }

    /// Retrieve Manifest Data
    pub async fn build(location: &Location) -> Result<Manifest, Box<dyn Error>> {
        let contents = match location {
            Location::Url(url) => Manifest::from_url(&url).await?,
            Location::FilePath(file_path) => Manifest::from_file(&file_path)?
        };

        let manifest: Manifest = serde_json::from_str(&contents)?;
        Ok(manifest)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Present,
    OutOfDate,
    Missing
}

#[derive(Debug, Clone)]
pub struct FileOperation<'a> {
    pub hash: String,
    pub patch_file: &'a PatchFile,
    pub size: u64,
    pub status: Status,
}

impl<'a> FileOperation<'a> {
    /// Process the manifest and return a list of file operations
    pub fn process(manifest: &Manifest) -> Vec<FileOperation> {
        manifest.files.iter().map(|file| {
            match fs::read(&file.path) {
                Ok(contents) => {
                    let digest = md5::compute(contents);
                    let digest_str = format!("{:x}", digest);
                    let new_size = std::fs::metadata(&file.path).unwrap_or_else(
                        |_| panic!("Failed to read metadata for file: {:?}", &file.path)
                    ).size();
                    FileOperation {
                        hash: file.hash.clone(),
                        status: if digest_str == file.hash {
                            Status::Present
                        } else {
                            Status::OutOfDate
                        },
                        patch_file: file,
                        size: new_size,
                    }
                }
                Err(_) => FileOperation {
                    hash: "".to_string(),
                    status: Status::Missing,
                    patch_file: file,
                    size: 0,
                }
            }
        }).collect()
    }
}


pub fn show_transaction_overview(manifest: &Manifest, operations: &Vec<FileOperation>) {
    println!("\nManifest Overview:");
    println!(" Version: {}", manifest.version);

    println!("\n {}", "Up-to-date files:".green());
    for op in operations.iter().filter(|op| op.status == Status::Present) {
        println!("  {} (Size: {})",
            op.patch_file.path.green(),
            humansize::format_size(op.size, BINARY)
        );
    }

    println!("\n {}", "Outdated files (will be updated):".yellow());
    for op in operations.iter().filter(|op| op.status == Status::OutOfDate) {
        println!("  {} (Current Size: {}, New Size: {})",
            op.patch_file.path.yellow(),
            humansize::format_size(op.size, BINARY),
            humansize::format_size(op.patch_file.size as u64, BINARY)
        );
    }

    println!("\n {}", "Missing files (will be downloaded):".red());
    for op in operations.iter().filter(|op| op.status == Status::Missing) {
        println!("  {} (New Size: {})",
            op.patch_file.path.red(),
            humansize::format_size(op.patch_file.size as u64, BINARY)
        );
    }

    println!("\nTransaction Summary:");
    println!(" Installing/Updating: {} files", operations.iter().filter(|x| x.status != Status::Present).count());

    // Calculate totals for non-present files
    let pending_ops: Vec<_> = operations.iter()
        .filter(|x| x.status != Status::Present)
        .collect();

    if !pending_ops.is_empty() {
        let total_download_size: i64 = pending_ops.iter()
            .map(|x| x.patch_file.size)
            .sum();

        let disk_space_change: i64 = pending_ops.iter()
            .map(|x| x.patch_file.size - x.size as i64)
            .sum();

        println!("\nTransaction Summary:");
        println!(" Installing/Updating: {} files", pending_ops.len());
        println!("\nTotal size of inbound files is {}. Need to download {}.",
            humansize::format_size(total_download_size as u64, BINARY),
            humansize::format_size(total_download_size as u64, BINARY)
        );

        if disk_space_change > 0 {
            println!("After this operation, {} of additional disk space will be used.",
                humansize::format_size(disk_space_change as u64, BINARY));
        } else {
            println!("After this operation, {} of disk space will be freed.",
                humansize::format_size(disk_space_change.abs() as u64, BINARY));
        }
    }
}