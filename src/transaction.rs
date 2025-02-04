use std::error::Error;

use colored::Colorize;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;
use humansize::{self, BINARY};

use super::manifest::{Manifest, PatchFile};
use super::Progress;

#[derive(PartialEq)]
enum Status {
    Present,
    OutOfDate,
    Missing
}

struct FileOperation<'a> {
    hash: String,
    patch_file: &'a PatchFile,
    size: u64,
    status: Status,
}

impl<'a> FileOperation<'a> {
    /// Process the manifest and return a list of file operations
    fn process(manifest: &Manifest) -> Vec<FileOperation> {
        manifest.files.iter().map(|file| {
            match std::fs::read(&file.path) {
                Ok(contents) => {
                    let digest = md5::compute(contents);
                    let digest_str = format!("{:x}", digest);
                    let new_size = std::fs::metadata(&file.path).unwrap_or_else(
                        |_| panic!("Failed to read metadata for file: {:?}", &file.path)
                    ).len();

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

pub struct Transaction<'a> {
    operations: Vec<FileOperation<'a>>,
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

    fn up_to_date(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status == Status::Present)
            .collect()
    }

    fn outdated(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status == Status::OutOfDate)
            .collect()
    }

    fn pending(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status != Status::Present)
            .collect()
    }

    fn missing(&self) -> Vec<&FileOperation> {
        self.operations.iter()
            .filter(|op| op.status == Status::Missing)
            .collect()
    }

    fn pending_count(&self) -> usize {
        self.operations.iter()
            .filter(|x| x.status != Status::Present)
            .count()
    }

    pub fn has_pending_operations(&self) -> bool {
        self.pending_count() > 0
    }

    fn total_download_size(&self) -> i64 {
        self.operations.iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| x.patch_file.size)
            .sum()
    }

    fn disk_space_change(&self) -> i64 {
        self.operations.iter()
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
                eprintln!("Failed to download {}: {}", &op.patch_file.url, response.status());
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
                    filename: dest_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                }.print();
            }
        }
        Ok(())
    }
}