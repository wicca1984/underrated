use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A rendered page containing the DOM, computed styles, and layout tree.
/// spec: S-13
pub struct Page {
    pub dom: Dom,
    pub styles: HashMap<NodeId, ComputedStyle>,
    pub layout: LayoutBox,
}

/// The main entry point for the engine: html + css -> Page.
/// spec: S-13
pub fn render(html: &str, css: &str, viewport_width: f32) -> Page {
    // 1. InputStream::from_utf8(html)
    let input = crate::encoding::InputStream::from_utf8(html.as_bytes());

    // 2. html::parse_document(input)
    let dom = crate::html::parse_document(input);

    // 3. css::parse_stylesheet(css)
    // spec: The actual API is in crate::css::parser::parse_stylesheet.
    let stylesheet = crate::css::parser::parse_stylesheet(css);

    // 4. style::compute_styles(&dom, &stylesheet)
    let styles = crate::style::compute_styles(&dom, &stylesheet);

    // 5. layout::layout_document(&dom, &styles, viewport_width)
    let layout = crate::layout::layout_document(&dom, &styles, viewport_width);

    // 6. return Page { dom, styles, layout }
    Page {
        dom,
        styles,
        layout,
    }
}

/// Renders content (html + css) all the way to a pixel canvas.
/// spec: S-13b
pub fn render_to_canvas(html: &str, css: &str, width: u32, height: u32) -> crate::raster::Canvas {
    // 1. let page = render(html, css, width as f32);
    let page = render(html, css, width as f32);

    // 2. let display_list = paint::build_display_list(&page.layout, &page.dom, &page.styles);
    let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);

    // 3. let canvas = raster::rasterize(&display_list, width, height);
    let canvas = crate::raster::rasterize(&display_list, width, height);

    // 4. return canvas.
    canvas
}

/// Renders HTML that contains its own `<style>` blocks (hoists them into the stylesheet).
/// spec: S-30
pub fn render_html(html: &str, viewport_width: f32) -> Page {
    // 1. InputStream::from_utf8(html)
    let input = crate::encoding::InputStream::from_utf8(html.as_bytes());

    // 2. html::parse_document(input)
    let dom = crate::html::parse_document(input);

    // 3. Walk DOM to collect the text of every <style> element (concatenate, in document order)
    // spec: In real browsers, head, style, script, etc. are display: none by default so they aren't rendered.
    let mut css_accumulator =
        String::from("head, style, script, meta, link, title { display: none; }\n");
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, .. }) = dom.data(node_id)
            && name.eq_ignore_ascii_case("style")
        {
            for &child_id in dom.children(node_id) {
                if let Some(NodeData::Text(text)) = dom.data(child_id) {
                    css_accumulator.push_str(text);
                }
            }
        }
    }

    // TODO(spec): Currently we do not filter or preprocess styles based on media or title attributes here.
    // TODO(spec): Text node content may contain HTML comment wrappers or CDATA; the CSS tokenizer handles them.

    // 4. css::parse_stylesheet(hoisted_css)
    let stylesheet = crate::css::parser::parse_stylesheet(&css_accumulator);

    // 5. style::compute_styles(&dom, &stylesheet)
    let styles = crate::style::compute_styles(&dom, &stylesheet);

    // 6. layout::layout_document(&dom, &styles, viewport_width)
    let layout = crate::layout::layout_document(&dom, &styles, viewport_width);

    // 7. return Page { dom, styles, layout }
    Page {
        dom,
        styles,
        layout,
    }
}

/// Renders HTML containing style blocks all the way to a pixel canvas.
/// spec: S-30
pub fn render_html_to_canvas(html: &str, width: u32, height: u32) -> crate::raster::Canvas {
    // 1. let page = render_html(html, width as f32);
    let page = render_html(html, width as f32);

    // 2. let display_list = paint::build_display_list(&page.layout, &page.dom, &page.styles);
    let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);

    // 3. let canvas = raster::rasterize(&display_list, width, height);
    let canvas = crate::raster::rasterize(&display_list, width, height);

    // 4. return canvas.
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeData;

    #[test]
    fn test_smoke_render() {
        let html = "<html><body><div></div></body></html>";
        let css = "div { width: 100px; height: 50px; }";
        let viewport_width = 800.0;

        let page = render(html, css, viewport_width);

        // Assert DOM contains the div
        let mut found_div = false;
        let doc = page.dom.document();
        for node_id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "div"
            {
                found_div = true;
                break;
            }
        }
        assert!(found_div, "DOM should contain a div element");

        // Assert layout root box width matches viewport within tolerance
        // root_box is Document node, it should have viewport_width
        assert!((page.layout.rect.size.width - viewport_width).abs() < 0.001);

        // Find the div in layout tree and check its size
        let mut div_box = None;
        let mut stack = vec![&page.layout];
        while let Some(b) = stack.pop() {
            if let Some(node_id) = b.node
                && let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "div"
            {
                div_box = Some(b);
                break;
            }
            for child in &b.children {
                stack.push(child);
            }
        }

        let b = div_box.expect("Layout tree should contain a box for the div");
        assert!((b.rect.size.width - 100.0).abs() < 0.001);
        assert!((b.rect.size.height - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_render_to_canvas() {
        let html = "<div></div>";
        let css = "div { background-color: #ff0000; width: 10px; height: 10px; }";
        let width = 20;
        let height = 20;

        let canvas = render_to_canvas(html, css, width, height);

        assert_eq!(canvas.width, 20);
        assert_eq!(canvas.height, 20);

        // Check a pixel that should be red
        // The div should be at (0,0) with 10x10 size.
        let pixel = canvas.pixel(5, 5);
        assert_eq!(pixel, 0xFFFF0000); // 0xAARRGGBB

        // Check a pixel outside the div
        let outside_pixel = canvas.pixel(15, 15);
        assert_eq!(outside_pixel, 0);
    }

    #[test]
    fn test_render_html() {
        let html = "<html><head><style>div { width: 100px; height: 50px; background-color: #ff0000; }</style></head><body><div></div></body></html>";
        let viewport_width = 800.0;

        let page = render_html(html, viewport_width);

        // Find the div in layout tree and check its size and styles
        let mut div_box = None;
        let mut stack = vec![&page.layout];
        while let Some(b) = stack.pop() {
            if let Some(node_id) = b.node
                && let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "div"
            {
                div_box = Some(b);
                break;
            }
            for child in &b.children {
                stack.push(child);
            }
        }

        let b = div_box.expect("Layout tree should contain a box for the div");
        assert!((b.rect.size.width - 100.0).abs() < 0.001);
        assert!((b.rect.size.height - 50.0).abs() < 0.001);

        // Check computed style for background-color
        let div_node_id = b.node.expect("Div layout box should have a DOM node");
        let style = page
            .styles
            .get(&div_node_id)
            .expect("Div should have computed style");
        if let Some(crate::css::values::CssValue::Color(c)) = style.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 0, 255));
        } else {
            panic!(
                "Expected background-color red, got {:?}",
                style.get("background-color")
            );
        }
    }

    #[test]
    fn test_render_html_to_canvas() {
        let html = "<html><head><style>div { background-color: #ff0000; width: 10px; height: 10px; }</style></head><body><div></div></body></html>";
        let width = 20;
        let height = 20;

        let canvas = render_html_to_canvas(html, width, height);

        assert_eq!(canvas.width, 20);
        assert_eq!(canvas.height, 20);

        // Check a pixel that should be red
        let pixel = canvas.pixel(5, 5);
        assert_eq!(pixel, 0xFFFF0000); // 0xAARRGGBB

        // Check a pixel outside the div
        let outside_pixel = canvas.pixel(15, 15);
        assert_eq!(outside_pixel, 0);
    }

    #[test]
    fn test_render_html_multiple_style_tags() {
        let html = "<html><head><style>div { width: 100px; }</style><style>div { height: 50px; background-color: #ff0000; }</style></head><body><div></div></body></html>";
        let viewport_width = 800.0;

        let page = render_html(html, viewport_width);

        // Find the div in layout tree and check its size
        let mut div_box = None;
        let mut stack = vec![&page.layout];
        while let Some(b) = stack.pop() {
            if let Some(node_id) = b.node
                && let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "div"
            {
                div_box = Some(b);
                break;
            }
            for child in &b.children {
                stack.push(child);
            }
        }

        let b = div_box.expect("Layout tree should contain a box for the div");
        assert!((b.rect.size.width - 100.0).abs() < 0.001);
        assert!((b.rect.size.height - 50.0).abs() < 0.001);

        // Check computed style for background-color
        let div_node_id = b.node.expect("Div layout box should have a DOM node");
        let style = page
            .styles
            .get(&div_node_id)
            .expect("Div should have computed style");
        if let Some(crate::css::values::CssValue::Color(c)) = style.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 0, 255));
        } else {
            panic!(
                "Expected background-color red, got {:?}",
                style.get("background-color")
            );
        }
    }
}
