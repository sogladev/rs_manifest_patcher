use std::io::Write;
use std::time::Duration;

use humansize::{format_size, DECIMAL};

const MAX_FILENAME_LENGTH: usize = 25;
const PROGRESS_BAR_WIDTH: usize = 20;
const TOTAL_LINE_WIDTH: usize = 80;

pub struct Progress {
    pub current: u64,
    pub total: u64,
    pub file_index: usize,
    pub total_files: usize,
    pub speed: f64,
    pub file_size: u64,
    pub elapsed: Duration,
    pub filename: String,
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
        let percent = (self.current as f64 / self.total as f64) * 100.0;
        let progress_bar = Self::create_progress_bar(self.current, self.total);
        let filename = Self::truncate_filename(&self.filename);
        let speed = format_size(self.speed as u64, DECIMAL);
        let size = format_size(self.file_size, DECIMAL);
        let total_files_width = self.total_files.to_string().len();

        if self.current >= self.total {
            print!("\r{:width$}", "", width = TOTAL_LINE_WIDTH); // Clear the line
            println!(
                "\r[{:>width$}/{}] {:<filename_width$} {} 100% (complete) {}         ",
                self.file_index,
                self.total_files,
                filename,
                progress_bar,
                size,
                width = total_files_width,
                filename_width = MAX_FILENAME_LENGTH - 1
            );
        } else {
            print!(
                "\r[{:>width$}/{}] {:<filename_width$} {} {:5.1}% {:<8}/s {}",
                self.file_index,
                self.total_files,
                filename,
                progress_bar,
                percent,
                speed,
                size,
                width = total_files_width,
                filename_width = MAX_FILENAME_LENGTH - 1
            );
            std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
        }
    }
}
