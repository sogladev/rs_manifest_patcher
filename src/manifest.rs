use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Location {
    Url(IpAddr),
    FilePath(PathBuf),
}