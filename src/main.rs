//! Binary entry point. Parses a small sample document with the `underrated`
//! engine and displays the rendered pixels in a native window.
//!
//! Run on a desktop with a display: `cargo run`. In a headless environment
//! (CI / dev container) no window can open; the engine itself is exercised by
//! the library tests.

use underrated::engine::render_to_canvas;
use underrated::shell::WinitWindow;

fn main() {
    // A tiny sample page so there is something to look at on screen.
    let html = "<!DOCTYPE html><html><body>\
        <div class=\"banner\"></div>\
        <div class=\"box\"></div>\
        <p>hello underrated</p>\
        </body></html>";
    let css = "body { margin: 0; } \
        .banner { width: 800px; height: 60px; background-color: rgb(40, 120, 220); } \
        .box { width: 300px; height: 150px; background-color: rgb(220, 70, 70); } \
        p { color: rgb(20, 140, 60); }";

    let width: u32 = 800;
    let height: u32 = 600;

    // Re-render every frame; the content is static for now.
    let window = WinitWindow::new("underrated", width, height);
    window.run(move || render_to_canvas(html, css, width, height));
}
