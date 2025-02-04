use std::fs::File;
use std::io::Write;
use std::env::temp_dir;

use rs_manifest_patcher::manifest::*;

#[test]
fn invalid_url() {
    let result = Location::parse("not-a-url".to_string());
    assert!(result.is_err());
}

#[test]
fn valid_url() {
    let result = Location::parse("127.0.0.1".to_string());
    assert!(result.is_ok());
}

#[test]
fn invalid_unix_path() {
    let result = Location::parse("/non/existent/path".to_string());
    assert!(result.is_err());
}

#[test]
fn invalid_windows_path() {
    let result = Location::parse("C://non//existent//file.txt".to_string());
    assert!(result.is_err());
}

#[test]
fn valid_file_path() {
    let mut temp_file_path = temp_dir();
    temp_file_path.push("manifest.json");

    // Create a temporary file
    let mut temp_file = File::create(&temp_file_path).expect("Failed to create temp file");
    writeln!(temp_file, "{{}}").expect("Failed to write to temp file");

    let result = Location::parse(temp_file_path.to_str().unwrap().to_string());
    assert!(result.is_ok());

    // Clean up the temporary file
    std::fs::remove_file(temp_file_path).expect("Failed to delete temp file");
}