use std::error::Error;
use std::path::PathBuf;

use colored::Colorize;
use futures::StreamExt;
use humansize::{self, BINARY};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use super::manifest::{Manifest, PatchFile};
use super::Progress;

#[derive(PartialEq, Clone)]
enum Status {
    Present,
    OutOfDate,
    Missing,
}

#[derive(Clone)]
struct FileOperation {
    patch_file: PatchFile,
    size: u64,
    status: Status,
}

impl FileOperation {
    /// Process the manifest and return a list of file operations
    fn process(manifest: &Manifest, base_path: &std::path::Path) -> Vec<FileOperation> {
        manifest
            .files
            .iter()
            .map(|file| {
                let full_path = base_path.join(&file.path);
                if !full_path.exists() {
                    return FileOperation {
                        status: Status::Missing,
                        patch_file: file.clone(),
                        size: 0,
                    };
                }

                match std::fs::read(&full_path) {
                    Ok(contents) => {
                        let digest = md5::compute(contents);
                        let digest_str = format!("{:x}", digest);
                        let new_size = std::fs::metadata(&full_path)
                            .unwrap_or_else(|_| {
                                panic!("Failed to read metadata for file: {:?}", &full_path)
                            })
                            .len();

                        FileOperation {
                            status: if digest_str == file.hash {
                                Status::Present
                            } else {
                                Status::OutOfDate
                            },
                            patch_file: file.clone(),
                            size: new_size,
                        }
                    }
                    Err(e) => {
                        panic!("Failed to read file {}: {}", full_path.display(), e);
                    }
                }
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct Transaction {
    operations: Vec<FileOperation>,
    manifest_version: String,
    base_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionReport {
    pub version: String,
    pub up_to_date_files: Vec<String>,
    pub outdated_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub total_download_size: u64,
    pub disk_space_change: i64,
    pub base_path: PathBuf,
}

impl Transaction {
    pub fn new(manifest: Manifest, base_path: PathBuf) -> Self {
        let operations = FileOperation::process(&manifest, &base_path);
        Transaction {
            operations,
            manifest_version: manifest.version,
            base_path,
        }
    }

    pub fn generate_report(&self) -> TransactionReport {
        TransactionReport {
            version: self.manifest_version.clone(),
            up_to_date_files: self
                .up_to_date()
                .iter()
                .map(|op| op.patch_file.path.clone())
                .collect(),
            outdated_files: self
                .outdated()
                .iter()
                .map(|op| op.patch_file.path.clone())
                .collect(),
            missing_files: self
                .missing()
                .iter()
                .map(|op| op.patch_file.path.clone())
                .collect(),
            total_download_size: self.total_download_size() as u64,
            disk_space_change: self.disk_space_change(),
            base_path: self.base_path.clone(),
        }
    }

    pub fn print(&self) {
        let report = self.generate_report();
        println!("\nManifest Overview:");
        println!(" Version: {}", report.version);
        println!(" Base path: {}", report.base_path.display());

        println!("\n {}", "Up-to-date files:".green());
        for file in report.up_to_date_files {
            println!("  {}", file.green());
        }

        println!("\n {}", "Outdated files (will be updated):".yellow());
        for file in report.outdated_files {
            println!("  {}", file.yellow());
        }

        println!("\n {}", "Missing files (will be downloaded):".red());
        for file in report.missing_files {
            println!("  {}", file.red());
        }

        if self.has_pending_operations() {
            println!("\nTransaction Summary:");
            println!(" Installing/Updating: {} files", self.pending_count());
            println!(
                "\nTotal size of inbound files is {}. Need to download {}.",
                humansize::format_size(report.total_download_size, BINARY),
                humansize::format_size(report.total_download_size, BINARY)
            );

            let disk_space_change = report.disk_space_change;
            if disk_space_change > 0 {
                println!(
                    "After this operation, {} of additional disk space will be used.",
                    humansize::format_size(disk_space_change as u64, BINARY)
                );
            } else {
                println!(
                    "After this operation, {} of disk space will be freed.",
                    humansize::format_size(disk_space_change.unsigned_abs(), BINARY)
                );
            }
        }
    }

    fn up_to_date(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::Present)
            .collect()
    }

    fn outdated(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::OutOfDate)
            .collect()
    }

    fn pending(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status != Status::Present)
            .collect()
    }

    fn missing(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::Missing)
            .collect()
    }

    fn pending_count(&self) -> usize {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .count()
    }

    pub fn has_pending_operations(&self) -> bool {
        self.pending_count() > 0
    }

    fn total_download_size(&self) -> i64 {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size)
            .sum()
    }

    fn disk_space_change(&self) -> i64 {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size - x.size as i64)
            .sum()
    }

    pub async fn download(&self) -> Result<(), Box<dyn Error>> {
        let http_client = reqwest::Client::new();
        for (idx, op) in self.pending().iter().enumerate() {
            let dest_path = std::path::Path::new(&op.patch_file.path);
            // Create parent directories if they don't exist
            if let Some(dir) = dest_path.parent() {
                tokio::fs::create_dir_all(dir).await?;
            }

            let response = http_client.get(&op.patch_file.url).send().await?;
            if !response.status().is_success() {
                eprintln!(
                    "Failed to download {}: {}",
                    &op.patch_file.url,
                    response.status()
                );
                continue;
            }

            let total_size = response.content_length().unwrap_or(0);
            let mut file = tokio::fs::File::create(dest_path).await?;
            let start = std::time::Instant::now();
            let mut downloaded: u64 = 0;

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;

                Progress {
                    current: downloaded,
                    total: total_size,
                    file_index: idx + 1,
                    total_files: self.pending_count(),
                    speed: downloaded as f64 / start.elapsed().as_secs_f64(),
                    file_size: total_size,
                    elapsed: start.elapsed(),
                    filename: dest_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                }
                .print();
            }
        }
        Ok(())
    }
}
