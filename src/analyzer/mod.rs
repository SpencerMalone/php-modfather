pub mod php_parser;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Context, Result};

/// Discovers PHP files in a directory
pub struct PhpFileDiscovery {
    pub paths: Vec<PathBuf>,
}

impl PhpFileDiscovery {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    /// Scan a directory for PHP files
    pub fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && self.is_php_file(path) {
                self.paths.push(path.to_path_buf());
            }
        }
        Ok(())
    }

    /// Check if a file is a PHP file based on extension
    fn is_php_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("php"))
            .unwrap_or(false)
    }

    /// Get all discovered PHP files
    pub fn get_files(&self) -> &[PathBuf] {
        &self.paths
    }
}

/// Read a file's content as a string
pub fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}
