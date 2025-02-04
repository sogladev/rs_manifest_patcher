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
    fn process(manifest: &Manifest) -> Vec<FileOperation> {
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

#[derive(Debug)]
pub struct Transaction<'a> {
    pub operations: Vec<FileOperation<'a>>,
    manifest: &'a Manifest,
}

impl<'a> Transaction<'a> {
    pub fn new(manifest: &'a Manifest) -> Self {
        let operations = FileOperation::process(manifest);
        Transaction {
            operations,
            manifest,
        }
    }

    pub fn print(&self) {
        println!("\nManifest Overview:");
        println!(" Version: {}", self.manifest.version);

        println!("\n {}", "Up-to-date files:".green());
        for op in self.up_to_date() {
            println!("  {} (Size: {})",
                op.patch_file.path.green(),
                humansize::format_size(op.size, BINARY)
            );
        }

        println!("\n {}", "Outdated files (will be updated):".yellow());
        for op in self.outdated() {
            println!("  {} (Current Size: {}, New Size: {})",
                op.patch_file.path.yellow(),
                humansize::format_size(op.size, BINARY),
                humansize::format_size(op.patch_file.size as u64, BINARY)
            );
        }

        println!("\n {}", "Missing files (will be downloaded):".red());
        for op in self.missing() {
            println!("  {} (New Size: {})",
                op.patch_file.path.red(),
                humansize::format_size(op.patch_file.size as u64, BINARY)
            );
        }

        if self.has_pending_operations() {
            println!("\nTransaction Summary:");
            println!(" Installing/Updating: {} files", self.pending_count());
            println!("\nTotal size of inbound files is {}. Need to download {}.",
                humansize::format_size(self.total_download_size() as u64, BINARY),
                humansize::format_size(self.total_download_size() as u64, BINARY)
            );

            let disk_space_change = self.disk_space_change();
            if disk_space_change > 0 {
                println!("After this operation, {} of additional disk space will be used.",
                    humansize::format_size(disk_space_change as u64, BINARY));
            } else {
                println!("After this operation, {} of disk space will be freed.",
                    humansize::format_size(disk_space_change.abs() as u64, BINARY));
            }
        }
    }

    pub fn up_to_date(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status == Status::Present)
            .collect()
    }

    pub fn outdated(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status == Status::OutOfDate)
            .collect()
    }

    pub fn missing(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status == Status::Missing)
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.operations.iter()
            .filter(|x| x.status != Status::Present)
            .count()
    }

    pub fn has_pending_operations(&self) -> bool {
        self.pending_count() > 0
    }

    pub fn total_download_size(&self) -> i64 {
        self.operations.iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size)
            .sum()
    }

    pub fn disk_space_change(&self) -> i64 {
        self.operations.iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size - x.size as i64)
            .sum()
    }

    pub async fn download(&self) -> Result<(), Box<dyn Error>> {
        let http_client = reqwest::Client::new();
        for op in self.operations.iter() {
            let dest_path = std::path::Path::new(&op.patch_file.path);
            if let Some(dir) = dest_path.parent() {
                tokio::fs::create_dir_all(dir).await?;
            }

            let response = http_client.get(&op.patch_file.url).send().await?;
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
}