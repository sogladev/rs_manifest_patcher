// use std::io::Write;
use std::time::Duration;

use humansize::{format_size, DECIMAL};

const MAX_FILENAME_LENGTH: usize = 20;
const PROGRESS_BAR_WIDTH: usize = 20;

#[derive(serde::Serialize)]
pub struct Progress {
    pub current: u64,
    pub file_index: usize,
    pub total_files: usize,
    pub speed: f64,
    pub file_size: u64,
    pub elapsed: Duration,
    pub filename: String,
    pub total_size_downloaded: u64,
    pub total_amount_left: u64,
    pub expected_time_left: f64,
    pub total_download_size: i64,
}

impl Progress {
    fn truncate_filename(name: &str) -> String {
        if name.len() <= MAX_FILENAME_LENGTH {
            format!("{:width$}", name, width = MAX_FILENAME_LENGTH)
        } else {
            format!("{}...", &name[..MAX_FILENAME_LENGTH - 3])
        }
    }

    fn create_progress_bar(current: u64, total: u64) -> String {
        let progress = current as f64 / total as f64;
        let filled = (progress * PROGRESS_BAR_WIDTH as f64) as usize;
        format!(
            "[{}{}]",
            "-".repeat(filled),
            " ".repeat(PROGRESS_BAR_WIDTH - filled)
        )
    }

    pub fn print(&self) {
        let percent = (self.current as f64 / self.file_size as f64) * 100.0;
        let progress_bar = Self::create_progress_bar(self.current, self.file_size);
        let filename = Self::truncate_filename(&self.filename);
        let speed = format_size(self.speed as u64, DECIMAL);
        let size = format_size(self.file_size, DECIMAL);
        let total_files_width = self.total_files.to_string().len();
        let total_left = format_size(self.total_amount_left, DECIMAL);

        if self.current >= self.file_size {
            print!("\r\x1B[2K"); // Clear the line
            println!(
                "\r[{:>width$}/{}] {:<filename_width$} {} 100% (complete) | {} | Left: {} | ETA: {:.1}s",
                self.file_index,
                self.total_files,
                filename,
                progress_bar,
                size,
                total_left,
                self.expected_time_left,
                width = total_files_width,
                filename_width = MAX_FILENAME_LENGTH - 1
            );
        } else {
            print!(
                "\r[{:>width$}/{}] {:<filename_width$} {} {:5.1}% | {:<8}/s | {} | Left: {} | ETA: {:.1}s",
                self.file_index,
                self.total_files,
                filename,
                progress_bar,
                percent,
                speed,
                size,
                total_left,
                self.expected_time_left,
                width = total_files_width,
                filename_width = MAX_FILENAME_LENGTH - 1
            );
            // std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
        }
    }
}
