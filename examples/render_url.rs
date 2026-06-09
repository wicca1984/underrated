//! Smoke test: fetch a real URL and render it through the full engine conductor.
//!
//! Usage: `cargo run --example render_url -- <url> [out.ppm]`
//! Prints pipeline stats and (optionally) writes a PPM screenshot.

use std::collections::BTreeSet;

use underrated::dom::NodeData;
use underrated::engine;
use underrated::loader::{HttpLoader, ResourceLoader};
use underrated::url::Url;

fn main() {
    let mut args = std::env::args().skip(1);
    let url_str = args
        .next()
        .unwrap_or_else(|| "https://example.com/".to_string());
    let out = args.next();

    let (width, height) = (1024u32, 768u32);
    let base = match Url::parse(&url_str) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("bad url: {e:?}");
            std::process::exit(1);
        }
    };

    let loader = HttpLoader;
    let bytes = match loader.load(&base) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("fetch failed: {e:?}");
            std::process::exit(2);
        }
    };
    let html = String::from_utf8_lossy(&bytes).into_owned();
    println!("fetched   : {} bytes from {url_str}", bytes.len());

    let page = engine::render_page(&html, &base, &loader, width as f32);

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

    let canvas = engine::render_page_to_canvas(&html, &base, &loader, width, height);
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

    if let Some(path) = out {
        let mut buf = format!("P3\n{} {}\n255\n", canvas.width, canvas.height).into_bytes();
        for p in &canvas.pixels {
            let (r, g, b) = ((p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF);
            buf.extend_from_slice(format!("{r} {g} {b}\n").as_bytes());
        }
        if std::fs::write(&path, buf).is_ok() {
            println!("screenshot: {path}");
        }
    }
}
