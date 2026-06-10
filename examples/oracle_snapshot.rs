//! Oracle differential-testing snapshot CLI example.
//!
//! This example renders a local HTML file and optional CSS file with the given
//! viewport dimensions, and outputs the normalized oracle snapshot JSON to stdout.
//!
//! Usage: `cargo run --example oracle_snapshot -- <html-file> [--css <css-file>] [--width <px>] [--height <px>]`

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!(
            "Usage: cargo run --example oracle_snapshot -- <html-file> [--css <css-file>] [--width <px>] [--height <px>]"
        );
        std::process::exit(2);
    }

    // The first argument is expected to be the html file path
    let html_file = &args[0];
    if html_file.starts_with('-') {
        eprintln!("Error: first argument must be the HTML file path, not an option flag.");
        eprintln!(
            "Usage: cargo run --example oracle_snapshot -- <html-file> [--css <css-file>] [--width <px>] [--height <px>]"
        );
        std::process::exit(2);
    }

    // Defaults
    let mut css_file: Option<String> = None;
    let mut width: u32 = 800;
    let mut height: u32 = 600;

    // Parse optional arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--css" => {
                if i + 1 < args.len() {
                    css_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --css option requires a file path argument.");
                    std::process::exit(2);
                }
            }
            "--width" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(val) => {
                            width = val;
                            i += 2;
                        }
                        Err(e) => {
                            eprintln!("Error: failed to parse width: {e}");
                            std::process::exit(2);
                        }
                    }
                } else {
                    eprintln!("Error: --width option requires a pixel value argument.");
                    std::process::exit(2);
                }
            }
            "--height" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(val) => {
                            height = val;
                            i += 2;
                        }
                        Err(e) => {
                            eprintln!("Error: failed to parse height: {e}");
                            std::process::exit(2);
                        }
                    }
                } else {
                    eprintln!("Error: --height option requires a pixel value argument.");
                    std::process::exit(2);
                }
            }
            other => {
                eprintln!("Error: unknown option '{other}'");
                eprintln!(
                    "Usage: cargo run --example oracle_snapshot -- <html-file> [--css <css-file>] [--width <px>] [--height <px>]"
                );
                std::process::exit(2);
            }
        }
    }

    // Read the files from disk
    let html = match std::fs::read_to_string(html_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading HTML file '{html_file}': {e}");
            std::process::exit(1);
        }
    };

    let css = match &css_file {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading CSS file '{path}': {e}");
                std::process::exit(1);
            }
        },
        None => String::new(),
    };

    // Export the oracle snapshot
    let snapshot = underrated::oracle::export_snapshot(&html, &css, width, height);

    // Format as pretty-printed JSON and write to stdout
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json_str) => {
            println!("{json_str}");
        }
        Err(e) => {
            eprintln!("Error serializing snapshot to JSON: {e}");
            std::process::exit(1);
        }
    }
}
