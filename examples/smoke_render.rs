//! Smoke render: offline rendering of a local HTML file.
//!
//! Usage: `cargo run --example smoke_render -- <html-file>`

use std::collections::BTreeSet;

use underrated::dom::NodeData;
use underrated::engine;
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
use underrated::url::Url;

/// A dummy loader that always returns NotFound.
/// We follow the pattern in oracle_snapshot/render_local_png for offline rendering of local HTML.
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
    let mut args = std::env::args().skip(1);
    let html_file = match args.next() {
        Some(file) => file,
        None => {
            eprintln!("Usage: cargo run --example smoke_render -- <html-file>");
            std::process::exit(2);
        }
    };

    // Read the HTML file from disk
    let html = match std::fs::read_to_string(&html_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading HTML file '{html_file}': {e}");
            std::process::exit(2);
        }
    };

    // Build base URL.
    let base_url = match Url::parse("http://localhost/") {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Error: failed to parse local base URL: {e:?}");
            std::process::exit(2);
        }
    };

    let (width, height) = (1024u32, 768u32);
    let loader = DummyLoader;

    let page = engine::render_page(&html, &base_url, &loader, width as f32);

    // DOM stats
    let doc = page.dom.document();
    let (mut elements, mut texts, mut links, mut imgs) = (0u32, 0u32, 0u32, 0u32);
    for id in page.dom.descendants(doc) {
        match page.dom.data(id) {
            Some(NodeData::Element { name, .. }) => {
                elements += 1;
                if name.eq_ignore_ascii_case("a") {
                    links += 1;
                }
                if name.eq_ignore_ascii_case("img") {
                    imgs += 1;
                }
            }
            Some(NodeData::Text(_)) => texts += 1,
            _ => {}
        }
    }
    println!("dom       : {elements} elements, {texts} text nodes, {links} <a>, {imgs} <img>");

    let canvas = engine::render_page_to_canvas(&html, &base_url, &loader, width, height);
    let total = canvas.pixels.len();
    let nonzero = canvas
        .pixels
        .iter()
        .filter(|&&p| (p & 0x00FF_FFFF) != 0)
        .count();
    let colors: BTreeSet<u32> = canvas.pixels.iter().map(|p| p & 0x00FF_FFFF).collect();
    println!(
        "raster    : {}x{} = {total} px, {nonzero} non-black ({:.1}%), {} distinct colors",
        canvas.width,
        canvas.height,
        100.0 * nonzero as f64 / total as f64,
        colors.len()
    );
}
