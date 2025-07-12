mod common;

use common::TempFile;
use rs_manifest_patcher::manifest::{Location, Manifest};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manifest_from_file() {
        // Prepare a temporary file with valid manifest JSON content

        let json_content = r#"
        {
            "Version": "1.0",
            "Uid": "5a63cd8c-956c-48a0-95ae-7e41d1e73182",
            "Files": [
                {
                    "Path": "files/A.bin",
                    "Hash": "b6d81b360a5672d80c27430f39153e2c",
                    "Size": 1048576,
                    "Custom": true,
                    "Urls": {
                        "cloudflare": "http://localhost:8080/files/A.bin",
                        "digitalocean": "http://localhost:8080/files/A.bin",
                        "none": "http://localhost:8080/files/A.bin"
                    }
                }
            ]
        }
        "#;
        let temp_file = TempFile::new("temp_manifest_valid.json", json_content);

        // Deserialize manifest from the file
        let location = Location::FilePath(temp_file.path.clone());
        let manifest = Manifest::build(&location)
            .await
            .expect("Failed to build manifest");
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, "files/A.bin");
    }

    #[cfg(test)]
    #[tokio::test]
    async fn test_manifest_deserialize_invalid_json() {
        // Prepare a temporary file with invalid JSON content

        let temp_file = common::TempFile::new("temp_manifest_invalid.json", "invalid json");

        // Expect Manifest::build to error out on invalid JSON
        let location = Location::FilePath(temp_file.path.clone());
        let result = Manifest::build(&location).await;
        assert!(result.is_err());
    }
}
