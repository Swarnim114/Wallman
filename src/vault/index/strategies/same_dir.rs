use crate::vault::index::indexing_strategy::IndexingStrategy;
use crate::vault::index::utils::has_matching_extension;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SameDirectoryStrategy;

impl IndexingStrategy for SameDirectoryStrategy {
    fn index(&self, target_path: &Path, extensions: &[String]) -> Vec<PathBuf> {
        let mut results = Vec::new();
        if let Ok(entries) = fs::read_dir(target_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && has_matching_extension(&path, extensions) {
                    results.push(path);
                }
            }
        }
        results
    }
}