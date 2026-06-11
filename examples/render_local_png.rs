//! Local HTML to PNG snapshot renderer.
//!
//! Renders a local HTML file through the shipping render path and saves it as a PNG.
//!
//! Usage: `cargo run --example render_local_png -- <html-file> [--width <px>] [--height <px>] [--out <png-path>]`

use underrated::engine;
use underrated::image;
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
use underrated::url::Url;

/// A dummy loader that always returns NotFound.
/// We follow the pattern in oracle_snapshot for offline rendering of local HTML.
struct DummyLoader;

impl ResourceLoader for DummyLoader {
    fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
        Err(LoadError::NotFound)
    }

    fn load_request(
        &self,
        _url: &Url,
        _method: HttpMethod,
        _body: &[u8],
        _content_type: Option<&str>,
    ) -> Result<LoaderResponse, LoadError> {
        Err(LoadError::NotFound)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!(
            "Usage: cargo run --example render_local_png -- <html-file> [--width <px>] [--height <px>] [--out <png-path>]"
        );
        std::process::exit(2);
    }

    // The first argument is expected to be the html file path
    let html_file = &args[0];
    if html_file.starts_with('-') {
        eprintln!("Error: first argument must be the HTML file path, not an option flag.");
        eprintln!(
            "Usage: cargo run --example render_local_png -- <html-file> [--width <px>] [--height <px>] [--out <png-path>]"
        );
        std::process::exit(2);
    }

    // Defaults
    let mut width: u32 = 800;
    let mut height: u32 = 600;
    let mut out_path: String = "var/render_local.png".to_string();

    // Parse optional arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
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
            "--out" => {
                if i + 1 < args.len() {
                    out_path = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --out option requires a file path argument.");
                    std::process::exit(2);
                }
            }
            other => {
                eprintln!("Error: unknown option '{other}'");
                eprintln!(
                    "Usage: cargo run --example render_local_png -- <html-file> [--width <px>] [--height <px>] [--out <png-path>]"
                );
                std::process::exit(2);
            }
        }
    }

    // Read the HTML file from disk
    let html = match std::fs::read_to_string(html_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading HTML file '{html_file}': {e}");
            std::process::exit(1);
        }
    };

    // Build base URL.
    // TODO(spec): If local resource loading is needed, use a file:// base URL with FsLoader.
    // For now, we follow oracle_snapshot's approach and use a placeholder localhost base URL with a DummyLoader.
    let base_url = match Url::parse("http://localhost/") {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Error: failed to parse local base URL: {e:?}");
            std::process::exit(1);
        }
    };

    // Render HTML page to pixel canvas
    let canvas = engine::render_page_to_canvas(&html, &base_url, &DummyLoader, width, height);

    // Encode canvas to PNG bytes
    let png_bytes = image::encode_png(&canvas);

    // Create the parent directory of the output path if it does not exist
    if let Some(parent) = std::path::Path::new(&out_path).parent()
        && !parent.exists()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!(
            "Error: failed to create directory {}: {}",
            parent.display(),
            e
        );
        std::process::exit(1);
    }

    // Write PNG bytes to output path
    if let Err(e) = std::fs::write(&out_path, &png_bytes) {
        eprintln!("Error writing PNG file '{out_path}': {e}");
        std::process::exit(1);
    }

    println!("screenshot: {}", out_path);
    println!(
        "stats     : {}x{} px, {} bytes",
        width,
        height,
        png_bytes.len()
    );
}
