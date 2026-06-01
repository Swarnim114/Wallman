use crate::vault::index::indexing_strategy::IndexingStrategy;
use crate::vault::index::utils::has_matching_extension;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct HomeDirectoryStrategy;

impl IndexingStrategy for HomeDirectoryStrategy {
    fn index(&self, _target_path: &Path, extensions: &[String]) -> Vec<PathBuf> {
        let mut results = Vec::new();
        if let Some(home_dir) = std::env::var_os("HOME").map(PathBuf::from) {
            for entry in WalkDir::new(home_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.into_path();
                if path.is_file() && has_matching_extension(&path, extensions) {
                    results.push(path);
                }
            }
        }
        results
    }
}