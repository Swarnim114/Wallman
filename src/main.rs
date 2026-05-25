mod vault;

use std::env;
use std::path::Path;
use vault::index::image_indexer::ImageIndexer;
use vault::index::strategies::{HomeDirectoryStrategy, RecursiveDirectoryStrategy, SameDirectoryStrategy};
use vault::index::strategy::IndexingStrategy;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: wallman <path> [ext1 ext2...] <strategy>");
        eprintln!("Strategies: rec, same, home");
        return;
    }
    
    let path_arg = &args[1];
    let strategy_arg = args.last().unwrap().as_str();
    
    // Everything between first arg (path) and last arg (strategy) are extensions
    let extensions = if args.len() > 3 {
        args[2..args.len() - 1].to_vec()
    } else {
        vec![]
    };

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
    
    // Optional: Print a few files to verify it works
    for (i, f) in files.iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, f.display());
    }
    if files.len() > 5 {
        println!("  ... and {} more", files.len() - 5);
    }
}
