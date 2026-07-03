use std::path::Path;
use crate::color::extractor::{ColorExtractingStrategy, ThemeMode};
use crate::color::strategies::native::NativeColorExtractor;

// usage: wallman colors -native <path-to-image>
//
// the -native flag picks which extraction engine to use.
// eventually -matugen and -pywal will live here too.
pub fn execute(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: wallman colors <strategy> <path-to-image>");
        eprintln!("Strategies: -native");
        eprintln!("Example:    wallman colors -native ~/wallpapers/mountain.png");
        return;
    }

    let strategy_flag = args[1].as_str();
    let image_path    = Path::new(&args[2]);

    // make sure the file actually exists before we try to open it
    if !image_path.exists() {
        eprintln!("Error: file not found — {}", image_path.display());
        return;
    }

    // pick the extractor based on the flag
    // Box<dyn ...> lets us treat all extractors the same way below,
    // even though they're different types — this is Rust's version of polymorphism
    let extractor: Box<dyn ColorExtractingStrategy> = match strategy_flag {
        "-native" => Box::new(NativeColorExtractor),
        other => {
            eprintln!("Unknown strategy: {}", other);
            eprintln!("Available strategies: -native");
            return;
        }
    };

    println!("Strategy   : {}", strategy_flag);
    println!("Image      : {}", image_path.display());
    println!();

    match extractor.extract(image_path, ThemeMode::Dark) {
        Ok(palette) => {
            let mode_label = match palette.mode {
                ThemeMode::Dark  => "Dark",
                ThemeMode::Light => "Light",
            };

            println!("Detected   : {} mode", mode_label);
            println!();

            // ── Surface Colors ─────────────────────────────────
            // these four form the "base layer" of a theme — backgrounds,
            // text, panels, subtext — like Catppuccin's base/surface/text/subtext
            println!("  Surface Colors");
            println!("  ──────────────────────────────────────────");
            print_surface_row("Background           ", &palette.background);
            print_surface_row("Secondary Background ", &palette.secondary_background);
            print_surface_row("Foreground           ", &palette.foreground);
            print_surface_row("Secondary Foreground ", &palette.secondary_foreground);
            println!();

            // ── Accent Palette ─────────────────────────────────
            println!("  #   Normal      Bright");
            println!("  ─────────────────────────");
            for i in 0..8 {
                println!(
                    "  {}   {}    {}",
                    i,
                    palette.colors[i],
                    palette.colors[i + 8]
                );
            }
            println!();

            print_swatches("Normal", &palette.colors[0..8]);
            print_swatches("Bright", &palette.colors[8..16]);
        }

        Err(e) => {
            eprintln!("Failed to extract colors: {}", e);
        }
    }
}

// prints one surface color row: label, hex value, and a small inline swatch
fn print_surface_row(label: &str, hex: &str) {
    if let Some((r, g, b)) = parse_hex(hex) {
        println!(
            "  {}  {}  \x1b[48;2;{};{};{}m   \x1b[0m",
            label, hex, r, g, b
        );
    }
}

fn print_swatches(label: &str, colors: &[String]) {
    print!("{}:  ", label);
    for hex in colors {
        if let Some((r, g, b)) = parse_hex(hex) {
            print!("\x1b[48;2;{};{};{}m   \x1b[0m", r, g, b);
        }
    }
    println!();
}

// parse "#RRGGBB" → (r, g, b)
// the ? operator returns None early if any step fails — no crash, just None
fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}
