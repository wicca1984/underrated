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

/// Renders HTML containing inline styles and/or external stylesheets, fetched via the loader.
/// spec: S-37
pub fn render_html_with_loader(
    html: &str,
    base: &crate::url::Url,
    loader: &dyn crate::loader::ResourceLoader,
    viewport_width: f32,
) -> Page {
    // 1. InputStream::from_utf8(html)
    let input = crate::encoding::InputStream::from_utf8(html.as_bytes());

    // 2. html::parse_document(input)
    let dom = crate::html::parse_document(input);

    // 3. Walk DOM to collect the text of every <style> element and fetch/decode every <link rel="stylesheet">
    // spec: In real browsers, head, style, script, etc. are display: none by default so they aren't rendered.
    let mut css_accumulator =
        String::from("head, style, script, meta, link, title { display: none; }\n");
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, attrs }) = dom.data(node_id) {
            if name.eq_ignore_ascii_case("style") {
                for &child_id in dom.children(node_id) {
                    if let Some(NodeData::Text(text)) = dom.data(child_id) {
                        css_accumulator.push_str(text);
                    }
                }
            } else if name.eq_ignore_ascii_case("link") {
                let mut is_stylesheet = false;
                let mut href = None;
                for (attr_name, attr_value) in attrs {
                    if attr_name.eq_ignore_ascii_case("rel") {
                        let has_stylesheet_rel = attr_value
                            .split_ascii_whitespace()
                            .any(|s| s.eq_ignore_ascii_case("stylesheet"));
                        if has_stylesheet_rel {
                            is_stylesheet = true;
                        }
                    } else if attr_name.eq_ignore_ascii_case("href") {
                        href = Some(attr_value);
                    }
                }
                if is_stylesheet && let Some(href_val) = href {
                    // Adjust href to bypass basic URL parser bug (treating relative path starting with letter as scheme)
                    let adjusted = if href_val.contains(':') {
                        let mut has_scheme = false;
                        if let Some(colon_pos) = href_val.find(':') {
                            let before_colon = &href_val[..colon_pos];
                            let path_or_query_pos =
                                href_val.find(['/', '?', '#']).unwrap_or(href_val.len());
                            if colon_pos < path_or_query_pos && !before_colon.is_empty() {
                                let mut chars = before_colon.chars();
                                if let Some(first) = chars.next()
                                    && first.is_ascii_alphabetic()
                                    && chars.all(|c| {
                                        c.is_ascii_alphanumeric()
                                            || c == '+'
                                            || c == '-'
                                            || c == '.'
                                    })
                                {
                                    has_scheme = true;
                                }
                            }
                        }
                        if has_scheme {
                            href_val.clone()
                        } else if href_val
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_alphabetic())
                        {
                            format!("./{}", href_val)
                        } else {
                            href_val.clone()
                        }
                    } else if href_val
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic())
                    {
                        format!("./{}", href_val)
                    } else {
                        href_val.clone()
                    };

                    // resolve relative to base
                    // spec: failed parse or failed fetch is ignored gracefully
                    if let Ok(resolved_url) = crate::url::Url::parse_with_base(&adjusted, base)
                        && let Ok(css_bytes) = loader.load(&resolved_url)
                        && let Ok(css_str) = String::from_utf8(css_bytes)
                    {
                        css_accumulator.push_str(&css_str);
                        css_accumulator.push('\n');
                    }
                }
            }
        }
    }

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

/// Renders a page with HTML, a base URL, a resource loader, and a viewport width.
/// This includes hoisting `<style>`, loading `<link rel="stylesheet">` CSS via resolved URLs,
/// running inline scripts to mutate the DOM, and resolving CSS using viewport-specific rules.
/// spec: S-64
pub fn render_page(
    html: &str,
    base_url: &crate::url::Url,
    loader: &dyn crate::loader::ResourceLoader,
    viewport_width: f32,
) -> Page {
    // 1. HTML -> DOM
    let input = crate::encoding::InputStream::from_utf8(html.as_bytes());
    let mut dom = crate::html::parse_document(input);

    // 2 & 3. Walk DOM to collect the text of every <style> element and fetch/decode every <link rel="stylesheet">
    let mut css_accumulator =
        String::from("head, style, script, meta, link, title { display: none; }\n");
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, attrs }) = dom.data(node_id) {
            if name.eq_ignore_ascii_case("style") {
                for &child_id in dom.children(node_id) {
                    if let Some(NodeData::Text(text)) = dom.data(child_id) {
                        css_accumulator.push_str(text);
                    }
                }
            } else if name.eq_ignore_ascii_case("link") {
                let mut is_stylesheet = false;
                let mut href = None;
                for (attr_name, attr_value) in attrs {
                    if attr_name.eq_ignore_ascii_case("rel") {
                        let has_stylesheet_rel = attr_value
                            .split_ascii_whitespace()
                            .any(|s| s.eq_ignore_ascii_case("stylesheet"));
                        if has_stylesheet_rel {
                            is_stylesheet = true;
                        }
                    } else if attr_name.eq_ignore_ascii_case("href") {
                        href = Some(attr_value);
                    }
                }
                if is_stylesheet && let Some(href_val) = href {
                    // Adjust href to bypass basic URL parser bug (treating relative path starting with letter as scheme)
                    let adjusted = if href_val.contains(':') {
                        let mut has_scheme = false;
                        if let Some(colon_pos) = href_val.find(':') {
                            let before_colon = &href_val[..colon_pos];
                            let path_or_query_pos =
                                href_val.find(['/', '?', '#']).unwrap_or(href_val.len());
                            if colon_pos < path_or_query_pos && !before_colon.is_empty() {
                                let mut chars = before_colon.chars();
                                if let Some(first) = chars.next()
                                    && first.is_ascii_alphabetic()
                                    && chars.all(|c| {
                                        c.is_ascii_alphanumeric()
                                            || c == '+'
                                            || c == '-'
                                            || c == '.'
                                    })
                                {
                                    has_scheme = true;
                                }
                            }
                        }
                        if has_scheme {
                            href_val.clone()
                        } else if href_val
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_alphabetic())
                        {
                            format!("./{}", href_val)
                        } else {
                            href_val.clone()
                        }
                    } else if href_val
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic())
                    {
                        format!("./{}", href_val)
                    } else {
                        href_val.clone()
                    };

                    if let Some(resolved_url) = crate::url::resolve(base_url, &adjusted)
                        && let Ok(css_bytes) = loader.load(&resolved_url)
                        && let Ok(css_str) = String::from_utf8(css_bytes)
                    {
                        css_accumulator.push_str(&css_str);
                        css_accumulator.push('\n');
                    }
                }
            }
        }
    }

    // 4. Run inline scripts
    dom = crate::script::run_inline_scripts(dom);

    // 5. Parse accumulated stylesheet
    let stylesheet = crate::css::parser::parse_stylesheet(&css_accumulator);

    // 6. Compute styles with viewport
    let styles = crate::style::compute_styles_with_viewport(&dom, &stylesheet, viewport_width);

    // 7. Layout document
    let layout = crate::layout::layout_document(&dom, &styles, viewport_width);

    Page {
        dom,
        styles,
        layout,
    }
}

/// Renders HTML containing inline styles, external stylesheets, and inline scripts to a pixel canvas.
/// spec: S-64
pub fn render_page_to_canvas(
    html: &str,
    base_url: &crate::url::Url,
    loader: &dyn crate::loader::ResourceLoader,
    width: u32,
    height: u32,
) -> crate::raster::Canvas {
    let page = render_page(html, base_url, loader, width as f32);
    let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);
    crate::raster::rasterize(&display_list, width, height)
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

    struct MockLoader {
        responses: HashMap<String, Result<Vec<u8>, crate::loader::LoadError>>,
    }

    impl crate::loader::ResourceLoader for MockLoader {
        fn load(&self, url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
            let serialized = url.serialize();
            if let Some(res) = self.responses.get(&serialized) {
                match res {
                    Ok(bytes) => Ok(bytes.clone()),
                    Err(crate::loader::LoadError::UnsupportedScheme) => {
                        Err(crate::loader::LoadError::UnsupportedScheme)
                    }
                    Err(crate::loader::LoadError::NotFound) => {
                        Err(crate::loader::LoadError::NotFound)
                    }
                    Err(crate::loader::LoadError::OutsideRoot) => {
                        Err(crate::loader::LoadError::OutsideRoot)
                    }
                    Err(crate::loader::LoadError::Io(s)) => {
                        Err(crate::loader::LoadError::Io(s.clone()))
                    }
                }
            } else {
                Err(crate::loader::LoadError::NotFound)
            }
        }
    }

    #[test]
    fn test_render_html_with_loader_basic() {
        use std::collections::HashMap;

        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/style.css".to_string(),
            Ok(b"div { background-color: #0000ff; }".to_vec()),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("https://example.com/").unwrap();
        let html = "<html><head><link rel=\"stylesheet\" href=\"style.css\"></head><body><div></div></body></html>";
        let page = render_html_with_loader(html, &base_url, &loader, 800.0);

        // Find div
        let doc = page.dom.document();
        let mut div_node_id = None;
        for node_id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "div"
            {
                div_node_id = Some(node_id);
                break;
            }
        }

        let div_id = div_node_id.expect("Should find a div node");
        let style = page
            .styles
            .get(&div_id)
            .expect("Should have computed style");
        if let Some(crate::css::values::CssValue::Color(c)) = style.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected blue background color");
        }
    }

    #[test]
    fn test_render_html_with_loader_graceful_failures() {
        use std::collections::HashMap;

        let mut responses = HashMap::new();
        // style1.css fails to load
        responses.insert(
            "https://example.com/style1.css".to_string(),
            Err(crate::loader::LoadError::NotFound),
        );
        // style2.css loads successfully
        responses.insert(
            "https://example.com/style2.css".to_string(),
            Ok(b"div { width: 150px; }".to_vec()),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("https://example.com/").unwrap();
        let html = "<html><head>\
            <link rel=\"stylesheet\" href=\"style1.css\">\
            <style>div { height: 120px; }</style>\
            <link rel=\"stylesheet\" href=\"style2.css\">\
            </head><body><div></div></body></html>";

        let page = render_html_with_loader(html, &base_url, &loader, 800.0);

        // Find div
        let doc = page.dom.document();
        let mut div_node_id = None;
        for node_id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "div"
            {
                div_node_id = Some(node_id);
                break;
            }
        }

        let div_id = div_node_id.expect("Should find a div node");
        let style = page
            .styles
            .get(&div_id)
            .expect("Should have computed style");

        // height should be 120px from the inline style
        if let Some(crate::css::values::CssValue::Length(val, _)) = style.get("height") {
            assert_eq!(*val, 120.0);
        } else {
            panic!("Expected height 120px");
        }

        // width should be 150px from style2.css
        if let Some(crate::css::values::CssValue::Length(val, _)) = style.get("width") {
            assert_eq!(*val, 150.0);
        } else {
            panic!("Expected width 150px");
        }
    }

    #[test]
    fn test_render_page_script_mutation_and_viewport_style() {
        let html = "
            <html>
              <head>
                <style>
                  #target { width: 100px; height: 50px; }
                  @media (max-width: 600px) {
                    #target { background-color: #ff0000; }
                  }
                  @media (min-width: 601px) {
                    #target { background-color: #0000ff; }
                  }
                </style>
              </head>
              <body>
                <div id=\"initial\"></div>
                <script>
                  let el = document.getElementById('initial');
                  el.setAttribute('id', 'target');
                </script>
              </body>
            </html>
        ";

        let loader = MockLoader {
            responses: HashMap::new(),
        };
        let base_url = crate::url::Url::parse("https://example.com/").unwrap();

        // Test with viewport <= 600px
        let page_narrow = render_page(html, &base_url, &loader, 500.0);
        let doc_narrow = page_narrow.dom.document();

        // The element with id initial should have been mutated to target
        let mut target_node_id = None;
        for id in page_narrow.dom.descendants(doc_narrow) {
            if let Some(NodeData::Element { attrs, .. }) = page_narrow.dom.data(id)
                && attrs.iter().any(|(k, v)| k == "id" && v == "target")
            {
                target_node_id = Some(id);
                break;
            }
        }
        let target_id = target_node_id.expect("Should find mutated element with id='target'");

        // Check computed style at 500px - should be #ff0000 (red)
        let style_narrow = page_narrow
            .styles
            .get(&target_id)
            .expect("Should have style");
        if let Some(crate::css::values::CssValue::Color(c)) = style_narrow.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 0, 255));
        } else {
            panic!("Expected red background color");
        }

        // Test with viewport > 600px
        let page_wide = render_page(html, &base_url, &loader, 800.0);
        let doc_wide = page_wide.dom.document();

        let mut target_node_id_wide = None;
        for id in page_wide.dom.descendants(doc_wide) {
            if let Some(NodeData::Element { attrs, .. }) = page_wide.dom.data(id)
                && attrs.iter().any(|(k, v)| k == "id" && v == "target")
            {
                target_node_id_wide = Some(id);
                break;
            }
        }
        let target_id_wide =
            target_node_id_wide.expect("Should find mutated element in wide viewport");

        // Check computed style at 800px - should be #0000ff (blue)
        let style_wide = page_wide
            .styles
            .get(&target_id_wide)
            .expect("Should have style");
        if let Some(crate::css::values::CssValue::Color(c)) = style_wide.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected blue background color");
        }
    }

    #[test]
    fn test_render_page_external_stylesheet_loading() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/responsive.css".to_string(),
            Ok(b"
                @media (max-width: 600px) {
                  p { background-color: #ff00ff; }
                }
            "
            .to_vec()),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("https://example.com/").unwrap();

        let html = "
            <html>
              <head>
                <link rel=\"stylesheet\" href=\"responsive.css\">
              </head>
              <body>
                <p>Hello world</p>
              </body>
            </html>
        ";

        // Under 500px, responsive.css applies and makes <p> magenta (#ff00ff)
        let page_narrow = render_page(html, &base_url, &loader, 500.0);
        let mut p_node_id = None;
        for id in page_narrow.dom.descendants(page_narrow.dom.document()) {
            if let Some(NodeData::Element { name, .. }) = page_narrow.dom.data(id)
                && name == "p"
            {
                p_node_id = Some(id);
                break;
            }
        }
        let p_id = p_node_id.expect("Should find p element");
        let style_narrow = page_narrow.styles.get(&p_id).expect("Should have style");
        if let Some(crate::css::values::CssValue::Color(c)) = style_narrow.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 255, 255));
        } else {
            panic!("Expected magenta background color for p element at 500px width");
        }

        // Under 800px, responsive.css's max-width: 600px does not apply
        let page_wide = render_page(html, &base_url, &loader, 800.0);
        let mut p_node_id_wide = None;
        for id in page_wide.dom.descendants(page_wide.dom.document()) {
            if let Some(NodeData::Element { name, .. }) = page_wide.dom.data(id)
                && name == "p"
            {
                p_node_id_wide = Some(id);
                break;
            }
        }
        let p_id_wide = p_node_id_wide.expect("Should find p element");
        let style_wide = page_wide.styles.get(&p_id_wide).expect("Should have style");
        assert!(style_wide.get("background-color").is_none());
    }

    #[test]
    fn test_render_page_to_canvas_smoke() {
        let html = "<html><head><style>div { background-color: #00ff00; width: 10px; height: 10px; }</style></head><body><div></div></body></html>";

        let loader = MockLoader {
            responses: HashMap::new(),
        };
        let base_url = crate::url::Url::parse("https://example.com/").unwrap();

        let canvas = render_page_to_canvas(html, &base_url, &loader, 20, 20);
        assert_eq!(canvas.width, 20);
        assert_eq!(canvas.height, 20);

        // Check if the 10x10 div is green (#00ff00 -> 0xFF00FF00)
        let pixel = canvas.pixel(5, 5);
        assert_eq!(pixel, 0xFF00FF00);
    }
}
