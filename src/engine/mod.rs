use crate::dom::Dom;
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
}
