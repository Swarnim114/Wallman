use crate::vault::index::strategy::IndexingStrategy;
use crate::vault::index::utils::has_allowed_extension;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct RecursiveDirectoryStrategy;

impl IndexingStrategy for RecursiveDirectoryStrategy {
    fn index(&self, target_path: &Path, extensions: &[String]) -> Vec<PathBuf> {
        let mut results = Vec::new();
        for entry in WalkDir::new(target_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.into_path();
            if path.is_file() && has_allowed_extension(&path, extensions) {
                results.push(path);
            }
        }
        results
    }
}