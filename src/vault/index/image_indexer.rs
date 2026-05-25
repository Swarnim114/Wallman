use crate::vault::index::strategy::IndexingStrategy;
use std::path::{Path, PathBuf};

pub struct ImageIndexer {
    strategy: Box<dyn IndexingStrategy>,
    allowed_extensions: Vec<String>,
}

impl ImageIndexer {
    pub fn new(strategy: Box<dyn IndexingStrategy>) -> Self {
        Self {
            strategy,
            allowed_extensions: Vec::new(),
        }
    }

    pub fn set_strategy(&mut self, strategy: Box<dyn IndexingStrategy>) {
        self.strategy = strategy;
    }

    pub fn set_filters(&mut self, extensions: Vec<String>) {
        self.allowed_extensions = extensions;
    }

    pub fn execute(&self, target_path: &Path) -> Vec<PathBuf> {
        self.strategy.index(target_path, &self.allowed_extensions)
    }
}
