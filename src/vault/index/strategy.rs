use std::path::{Path, PathBuf};

pub trait IndexingStrategy {
    fn index(&self, target_path: &Path, extensions: &[String]) -> Vec<PathBuf>;
}