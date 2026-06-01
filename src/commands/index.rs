use std::path::Path;
use crate::vault::index::image_indexer::ImageIndexer;
use crate::vault::index::strategies::{HomeDirectoryStrategy, RecursiveDirectoryStrategy, SameDirectoryStrategy};
use crate::vault::index::indexing_strategy::IndexingStrategy;
use crate::vault::index::utils;

pub fn execute(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: wallman index <path> [ext1 ext2...] <strategy>");
        eprintln!("Strategies: rec, same, home");
        return;
    }
    
    let path_arg = &args[1];
    let strategy_arg = args.last().unwrap().as_str();
    
    // Everything between first arg (path) and last arg (strategy) are extensions
    let all_extensions = if args.len() > 3 {
        args[2..args.len() - 1].to_vec()
    } else {
        vec![]
    };

    let mut extensions = Vec::new();
    for ext in all_extensions {
        if utils::is_image_extension(&ext) {
            extensions.push(ext);
        }
    }

    let strategy: Box<dyn IndexingStrategy> = match strategy_arg {
        "rec" => Box::new(RecursiveDirectoryStrategy),
        "home" => Box::new(HomeDirectoryStrategy),
        "same" | _ => Box::new(SameDirectoryStrategy),
    };

    let mut indexer = ImageIndexer::new(strategy);
    indexer.set_filters(extensions.clone());
    
    let target = Path::new(path_arg);
    let files = indexer.execute(target);
    
    println!("Target Path: {}", path_arg);
    println!("Strategy: {}", strategy_arg);
    println!("Extensions allowed: {:?}", extensions);
    println!("Found {} images.", files.len());
    
    for (i, f) in files.iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, f.display());
    }
    if files.len() > 5 {
        println!("  ... and {} more", files.len() - 5);
    }
}
