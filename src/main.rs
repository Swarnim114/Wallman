mod vault;
mod commands;
mod color;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: wallman <command> [args...]");
        eprintln!("Commands: index");
        return;
    }

    let command = args[1].as_str();
    
    // We pass args[1..] so that the actual command sees its arguments starting at index 0 or 1
    // Let's pass args[1..] so index 0 is the command name ("index"), index 1 is the path, etc.
    let command_args = &args[1..];

    match command {
        "index" => commands::index::execute(command_args),
        _ => eprintln!("Unknown command: {}. Available commands: index", command),
    }
}
