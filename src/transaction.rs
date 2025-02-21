use std::error::Error;
use std::path::PathBuf;

use colored::Colorize;
use futures::StreamExt;
use humansize::BINARY;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use super::manifest::{Manifest, PatchFile};
use super::Progress;

#[derive(PartialEq, Clone)]
pub enum Status {
    Present,
    OutOfDate,
    Missing,
}

#[derive(Clone)]
/// Represents a transaction operation involving file patching.
///
/// This struct holds the details for an operation that patches a file, including the patch file data,
/// the size of the file patch, and the current status of the operation.
///
/// # Fields
/// - `patch_file`: The patch file associated with the operation.
/// - `size`: The size of the patch, represented as a 64-bit signed integer.
/// - `status`: The status of the file operation.
pub struct FileOperation {
    pub patch_file: PatchFile,
    pub size: i64,
    pub status: Status,
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
                        let new_size: i64 = std::fs::metadata(&full_path)
                            .unwrap_or_else(|_| {
                                panic!("Failed to read metadata for file: {:?}", &full_path)
                            })
                            .len()
                            .try_into()
                            .unwrap();

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
    pub base_path: PathBuf,
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
        for op in self.up_to_date() {
            println!(
                "  {} (Size: {})",
                op.patch_file.path.green(),
                humansize::format_size(op.size as u64, BINARY)
            );
        }

        println!("\n {}", "Outdated files (will be updated):".yellow());
        for op in self.outdated() {
            println!(
                "  {} (Current Size: {}, New Size: {})",
                op.patch_file.path.yellow(),
                humansize::format_size(op.size as u64, BINARY),
                humansize::format_size(op.patch_file.size as u64, BINARY)
            );
        }

        println!("\n {}", "Missing files (will be downloaded):".red());
        for op in self.missing() {
            println!(
                "  {} (New Size: {})",
                op.patch_file.path.red(),
                humansize::format_size(op.patch_file.size as u64, BINARY)
            );
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

    pub fn pending(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status != Status::Present)
            .collect()
    }

    pub fn missing(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::Missing)
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .count()
    }

    pub fn has_pending_operations(&self) -> bool {
        self.pending_count() > 0
    }

    pub fn total_download_size(&self) -> i64 {
        let total = self
            .operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size)
            .sum();
        assert!(
            total >= 0,
            "Total download size must be non-negative, but found {}.",
            total
        );
        total
    }

    fn disk_space_change(&self) -> i64 {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size - x.size)
            .sum()
    }

    pub async fn download<F>(&self, progress_handler: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&Progress) -> Result<(), Box<dyn Error>> + Send + 'static,
    {
        let http_client = reqwest::Client::new();
        let mut total_size_downloaded = 0;
        let total_download_size = self.total_download_size();
        for (idx, op) in self.pending().iter().enumerate() {
            // Create parent directories if they don't exist
            let dest_path = self.base_path.join(&op.patch_file.path);
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

            let file_size = op.patch_file.size;
            let mut file = tokio::fs::File::create(dest_path.clone()).await?;
            let start = std::time::Instant::now();
            let mut downloaded: u64 = 0;

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| e.to_string())?;
                file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                downloaded += chunk.len() as u64;
                total_size_downloaded += chunk.len() as u64;

                // Handle potential underflow
                let total_amount_left =
                    (total_download_size as u64).saturating_sub(total_size_downloaded);

                // Compute download speed and expected time left
                let speed = downloaded as f64 / start.elapsed().as_secs_f64();
                let expected_time_left = if speed > 0.0 {
                    // Compute remaining time and cap at, say, 24 hours (86400 s).
                    (total_amount_left as f64 / speed).min(86400.0)
                } else {
                    0.0
                };

                let progress = Progress {
                    current: downloaded,
                    file_index: idx + 1,
                    total_files: self.pending_count(),
                    speed,
                    file_size: file_size.try_into().unwrap(),
                    elapsed: start.elapsed(),
                    filename: dest_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),

                    total_size_downloaded,
                    total_amount_left,
                    expected_time_left,
                    total_download_size,
                };

                progress_handler(&progress)?;
            }
        }
        Ok(())
    }
}
