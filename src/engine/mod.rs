use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::loader::ResourceLoader;
use crate::style::ComputedStyle;
use crate::url::Url;
use std::collections::HashMap;

/// A rendered page containing the DOM, computed styles, and layout tree.
/// spec: S-13
pub struct Page {
    pub dom: Dom,
    pub styles: HashMap<NodeId, ComputedStyle>,
    pub layout: LayoutBox,
}

// spec: S-79
// The default User-Agent stylesheet used by the browser.
const UA_DEFAULT_CSS: &str = "\
html, body { background: #fff; background-color: #fff; color: #000; }\n\
body { margin: 8px; }\n\
div, p, h1, h2, h3, h4, h5, h6, ul, ol, li, section, header, footer, nav, article { display: block; }\n\
p { margin-top: 1em; margin-bottom: 1em; }\n\
h1 { margin-top: 0.67em; margin-bottom: 0.67em; font-weight: bold; }\n\
h2 { margin-top: 0.83em; margin-bottom: 0.83em; font-weight: bold; }\n\
h3 { margin-top: 1em; margin-bottom: 1em; font-weight: bold; }\n\
h4 { margin-top: 1.33em; margin-bottom: 1.33em; font-weight: bold; }\n\
h5 { margin-top: 1.67em; margin-bottom: 1.67em; font-weight: bold; }\n\
h6 { margin-top: 2.33em; margin-bottom: 2.33em; font-weight: bold; }\n\
a { color: #0000ee; text-decoration: underline; }\n\
b, strong { font-weight: bold; }\n\
i, em { font-style: italic; }\n\
head, style, script, meta, link, title { display: none; }\n\
";

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
    let mut css_accumulator = String::from(UA_DEFAULT_CSS);
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

fn load_image_safely_with_loader(
    loader: &dyn crate::loader::ResourceLoader,
    src: &str,
    base_url: Option<&crate::url::Url>,
) -> Option<Vec<u8>> {
    if src.starts_with("data:") {
        return crate::loader::load_data_uri(src);
    }

    let resolved_url = if let Some(base) = base_url {
        crate::url::resolve(base, src)
    } else {
        crate::url::Url::parse(src).ok()
    };

    if let Some(url) = resolved_url {
        if url.scheme == "data" {
            return crate::loader::load_data_uri(&url.serialize());
        }

        if url.scheme == "http" || url.scheme == "https" {
            if let Ok(bytes) = loader.load(&url) {
                return Some(bytes);
            }
            return None;
        }

        return crate::loader::load_image_safely(src, base_url);
    }

    None
}

fn fetch_and_decode_images(
    dom: &crate::dom::Dom,
    base_url: &crate::url::Url,
    loader: &dyn crate::loader::ResourceLoader,
) {
    let mut effective_base = base_url.clone();
    let doc = dom.document();
    for n_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name: el_name, .. }) = dom.data(n_id)
            && el_name.eq_ignore_ascii_case("base")
            && let Some(href) = dom.get_attribute(n_id, "href")
        {
            if let Some(resolved) = crate::url::resolve(base_url, href) {
                effective_base = resolved;
            }
            break;
        }
    }

    for n_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, .. }) = dom.data(n_id)
            && name.eq_ignore_ascii_case("img")
            && let Some(src) = dom.get_attribute(n_id, "src")
            && let Some(bytes) = load_image_safely_with_loader(loader, src, Some(&effective_base))
            && let Some(decoded) = crate::image::decode_png(&bytes)
        {
            dom.add_image(src.to_string(), decoded);
        }
    }
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
    let mut css_accumulator = String::from(UA_DEFAULT_CSS);
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

    fetch_and_decode_images(&dom, base, loader);

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
    let mut css_accumulator = String::from(UA_DEFAULT_CSS);
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

    fetch_and_decode_images(&dom, base_url, loader);

    Page {
        dom,
        styles,
        layout,
    }
}

/// Navigates to a requested page from a form submission or similar action.
/// Resolves forms::Method -> loader::HttpMethod, resolves req.url against base,
/// loads request via load_request, resolves charset, and runs the existing render_page pipeline.
/// On failure, returns an empty page.
/// spec: S-89
pub fn navigate(
    req: &crate::forms::NavigationRequest,
    base: &Url,
    loader: &dyn ResourceLoader,
    viewport_width: f32,
) -> Page {
    let resolved_url = match crate::url::resolve(base, &req.url) {
        Some(url) => url,
        None => return render_page("", base, loader, viewport_width),
    };

    let method = match req.method {
        crate::forms::Method::Get => crate::loader::HttpMethod::Get,
        crate::forms::Method::Post => crate::loader::HttpMethod::Post,
    };

    let response = match loader.load_request(
        &resolved_url,
        method,
        req.body.as_bytes(),
        req.content_type.as_deref(),
    ) {
        Ok(res) => res,
        Err(_) => return render_page("", base, loader, viewport_width),
    };

    let charset = crate::encoding::sniff_charset(&response.bytes, response.charset.as_deref());
    let mut offset = 0;
    if response.bytes.starts_with(&[0xEF, 0xBB, 0xBF]) && charset == crate::encoding::Charset::Utf8
    {
        offset = 3;
    } else if (response.bytes.starts_with(&[0xFE, 0xFF])
        && charset == crate::encoding::Charset::Utf16Be)
        || (response.bytes.starts_with(&[0xFF, 0xFE])
            && charset == crate::encoding::Charset::Utf16Le)
    {
        offset = 2;
    }
    let decoded_html = crate::encoding::decode(&response.bytes[offset..], charset);

    render_page(&decoded_html, &resolved_url, loader, viewport_width)
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
        let html = "<html><head><style>html, body { background-color: transparent; } body { margin: 0; } div { background-color: #ff0000; width: 10px; height: 10px; }</style></head><body><div></div></body></html>";
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

        fn load_request(
            &self,
            url: &crate::url::Url,
            method: crate::loader::HttpMethod,
            body: &[u8],
            content_type: Option<&str>,
        ) -> Result<crate::loader::LoaderResponse, crate::loader::LoadError> {
            let method_prefix = match method {
                crate::loader::HttpMethod::Get => "GET",
                crate::loader::HttpMethod::Post => "POST",
            };
            let lookup_key = format!(
                "{}:{}|body:{:?}|ct:{:?}",
                method_prefix,
                url.serialize(),
                body,
                content_type
            );
            if let Some(res) = self.responses.get(&lookup_key) {
                let bytes = match res {
                    Ok(b) => b.clone(),
                    Err(e) => {
                        return Err(match e {
                            crate::loader::LoadError::UnsupportedScheme => {
                                crate::loader::LoadError::UnsupportedScheme
                            }
                            crate::loader::LoadError::NotFound => {
                                crate::loader::LoadError::NotFound
                            }
                            crate::loader::LoadError::OutsideRoot => {
                                crate::loader::LoadError::OutsideRoot
                            }
                            crate::loader::LoadError::Io(s) => {
                                crate::loader::LoadError::Io(s.clone())
                            }
                        });
                    }
                };
                return Ok(crate::loader::LoaderResponse {
                    bytes,
                    content_type: "text/html".to_string(),
                    charset: Some("utf-8".to_string()),
                });
            }

            self.load_rich(url)
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
        let html = "<html><head><style>html, body { background-color: transparent; } body { margin: 0; } div { background-color: #00ff00; width: 10px; height: 10px; }</style></head><body><div></div></body></html>";

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

    #[test]
    fn test_ua_default_stylesheet() {
        let html =
            "<html><body><p>Hello</p><a href=\"#\">Link</a><b>Bold</b><i>Italic</i></body></html>";
        let page = render_html(html, 800.0);

        // Find elements and check their resolved styles
        let doc = page.dom.document();
        let mut p_style = None;
        let mut a_style = None;
        let mut b_style = None;
        let mut i_style = None;
        let mut body_style = None;

        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(id) {
                match name.as_str() {
                    "p" => p_style = page.styles.get(&id),
                    "a" => a_style = page.styles.get(&id),
                    "b" => b_style = page.styles.get(&id),
                    "i" => i_style = page.styles.get(&id),
                    "body" => body_style = page.styles.get(&id),
                    _ => {}
                }
            }
        }

        let p_s = p_style.expect("p should have styles");
        assert_eq!(
            p_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );
        // p should have margin-top and margin-bottom 1em
        assert_eq!(
            p_s.get("margin-top"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );

        let a_s = a_style.expect("a should have styles");
        if let Some(crate::css::values::CssValue::Color(c)) = a_s.get("color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(0, 0, 0xee, 255));
        } else {
            panic!("Expected color blue #0000ee for link");
        }
        assert_eq!(
            a_s.get("text-decoration"),
            Some(&crate::css::values::CssValue::Keyword(
                "underline".to_string()
            ))
        );

        let b_s = b_style.expect("b should have styles");
        assert_eq!(
            b_s.get("font-weight"),
            Some(&crate::css::values::CssValue::Keyword("bold".to_string()))
        );

        let i_s = i_style.expect("i should have styles");
        assert_eq!(
            i_s.get("font-style"),
            Some(&crate::css::values::CssValue::Keyword("italic".to_string()))
        );

        let body_s = body_style.expect("body should have styles");
        assert_eq!(
            body_s.get("margin-top"),
            Some(&crate::css::values::CssValue::Length(
                8.0,
                crate::css::values::LengthUnit::Px
            ))
        );
        if let Some(crate::css::values::CssValue::Color(c)) = body_s.get("background-color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 255, 255, 255));
        } else {
            panic!("Expected white background color for body");
        }
    }

    #[test]
    fn test_navigate_get_and_post_forms() {
        use std::collections::HashMap;

        // Base URL
        let base_url = Url::parse("https://example.com/").unwrap();

        // Prepare MockLoader with responses for GET and POST searches
        let mut responses = HashMap::new();
        // GET request URL: https://example.com/search?q=rust
        responses.insert(
            "https://example.com/search?q=rust".to_string(),
            Ok(b"<html><body><h1>Search results: rust (GET)</h1></body></html>".to_vec()),
        );

        // POST request key: "POST:https://example.com/search|body:[113, 61, 114, 117, 115, 116]|ct:Some(\"application/x-www-form-urlencoded\")"
        let post_body = b"q=rust";
        let post_key = format!(
            "POST:https://example.com/search|body:{:?}|ct:Some(\"application/x-www-form-urlencoded\")",
            post_body
        );
        responses.insert(
            post_key,
            Ok(b"<html><body><h1>Search results: rust (POST)</h1></body></html>".to_vec()),
        );

        let loader = MockLoader { responses };

        // Test GET navigation directly
        let get_request = crate::forms::NavigationRequest {
            url: "/search?q=rust".to_string(),
            method: crate::forms::Method::Get,
            body: String::new(),
            content_type: None,
        };

        let page_get = navigate(&get_request, &base_url, &loader, 800.0);

        // Assert DOM contains the header with search results
        let mut found_get_text = false;
        let doc_get = page_get.dom.document();
        for id in page_get.dom.descendants(doc_get) {
            if let Some(NodeData::Text(text)) = page_get.dom.data(id)
                && text.contains("Search results: rust (GET)")
            {
                found_get_text = true;
            }
        }
        assert!(
            found_get_text,
            "GET page should contain search results text"
        );

        // Test POST navigation directly
        let post_request = crate::forms::NavigationRequest {
            url: "/search".to_string(),
            method: crate::forms::Method::Post,
            body: "q=rust".to_string(),
            content_type: Some("application/x-www-form-urlencoded".to_string()),
        };

        let page_post = navigate(&post_request, &base_url, &loader, 800.0);

        // Assert DOM contains the header with search results
        let mut found_post_text = false;
        let doc_post = page_post.dom.document();
        for id in page_post.dom.descendants(doc_post) {
            if let Some(NodeData::Text(text)) = page_post.dom.data(id)
                && text.contains("Search results: rust (POST)")
            {
                found_post_text = true;
            }
        }
        assert!(
            found_post_text,
            "POST page should contain search results text"
        );

        // Let's create actual form structures in DOM and submit them to get NavigationRequests
        let mut dom_get = Dom::new();
        fn create_el(
            dom: &mut Dom,
            tag_name: &str,
            attrs: &[(&str, &str)],
        ) -> crate::infra::NodeId {
            let attributes = attrs
                .iter()
                .map(|&(k, v)| (k.to_string(), v.to_string()))
                .collect();
            dom.create_node(NodeData::Element {
                name: tag_name.to_string(),
                attrs: attributes,
            })
        }

        // Create a GET Form
        let form_get = create_el(
            &mut dom_get,
            "form",
            &[("action", "/search"), ("method", "get")],
        );
        let input_get = create_el(
            &mut dom_get,
            "input",
            &[("type", "text"), ("name", "q"), ("value", "rust")],
        );
        dom_get.append_child(form_get, input_get);

        let req_get =
            crate::forms::submit(&dom_get, form_get, &crate::forms::FormState::new()).unwrap();
        // Check that navigate successfully resolves and loads this request
        let page_get_submitted = navigate(&req_get, &base_url, &loader, 800.0);
        let mut found_get_submitted_text = false;
        for id in page_get_submitted
            .dom
            .descendants(page_get_submitted.dom.document())
        {
            if let Some(NodeData::Text(text)) = page_get_submitted.dom.data(id)
                && text.contains("Search results: rust (GET)")
            {
                found_get_submitted_text = true;
            }
        }
        assert!(
            found_get_submitted_text,
            "Submitted GET page should contain search results text"
        );

        // Create a POST Form
        let mut dom_post = Dom::new();
        let form_post = create_el(
            &mut dom_post,
            "form",
            &[("action", "/search"), ("method", "post")],
        );
        let input_post = create_el(
            &mut dom_post,
            "input",
            &[("type", "text"), ("name", "q"), ("value", "rust")],
        );
        dom_post.append_child(form_post, input_post);

        let req_post =
            crate::forms::submit(&dom_post, form_post, &crate::forms::FormState::new()).unwrap();
        // Check that navigate successfully resolves and loads this request
        let page_post_submitted = navigate(&req_post, &base_url, &loader, 800.0);
        let mut found_post_submitted_text = false;
        for id in page_post_submitted
            .dom
            .descendants(page_post_submitted.dom.document())
        {
            if let Some(NodeData::Text(text)) = page_post_submitted.dom.data(id)
                && text.contains("Search results: rust (POST)")
            {
                found_post_submitted_text = true;
            }
        }
        assert!(
            found_post_submitted_text,
            "Submitted POST page should contain search results text"
        );
    }

    #[test]
    fn test_remote_img_fetch_decode_blit_s90() {
        use crate::paint::DisplayItem;
        use crate::raster::Canvas;

        fn encode_base64(bytes: &[u8]) -> String {
            const CHARS: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut result = String::new();
            let mut i = 0;
            while i < bytes.len() {
                let b0 = bytes[i];
                let b1 = if i + 1 < bytes.len() {
                    Some(bytes[i + 1])
                } else {
                    None
                };
                let b2 = if i + 2 < bytes.len() {
                    Some(bytes[i + 2])
                } else {
                    None
                };

                let val0 = b0 >> 2;
                let val1 = ((b0 & 3) << 4) | (b1.unwrap_or(0) >> 4);
                let val2 = b1.map(|b| ((b & 15) << 2) | (b2.unwrap_or(0) >> 6));
                let val3 = b2.map(|b| b & 63);

                result.push(CHARS[val0 as usize] as char);
                result.push(CHARS[val1 as usize] as char);
                if let Some(v2) = val2 {
                    result.push(CHARS[v2 as usize] as char);
                } else {
                    result.push('=');
                }
                if let Some(v3) = val3 {
                    result.push(CHARS[v3 as usize] as char);
                } else {
                    result.push('=');
                }

                i += 3;
            }
            result
        }

        // 1. Generate a valid 2x2 PNG image bytes
        let mut source_canvas = Canvas::new(2, 2);
        // 0xAARRGGBB
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        source_canvas.pixels[2] = 0xFF0000FF; // Blue
        source_canvas.pixels[3] = 0xFFFFFF00; // Yellow
        let png_bytes = crate::image::encode_png(&source_canvas);

        // 2. Set up MockLoader to return these PNG bytes for our http image URL
        let image_url = "http://example.com/image.png";
        let mut responses = HashMap::new();
        responses.insert(image_url.to_string(), Ok(png_bytes.clone()));

        // Also we will have a data: URI in the HTML
        let data_uri = format!("data:image/png;base64,{}", encode_base64(&png_bytes));

        let loader = MockLoader { responses };

        // 3. Render HTML containing both http image URL and data: URI
        let html = format!(
            r#"<html>
                <body>
                    <img id="img1" src="http://example.com/image.png" style="width: 4px; height: 4px;">
                    <img id="img2" src="{}" style="width: 4px; height: 4px;">
                </body>
            </html>"#,
            data_uri
        );

        let base_url = crate::url::Url::parse("http://example.com/").unwrap();
        let page = render_page(&html, &base_url, &loader, 800.0);

        // 4. Build DisplayList and verify it contains DisplayItem::Image items with decoded DecodedImage
        let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);
        let items = display_list.0;

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(
            image_items.len(),
            2,
            "Should have 2 image display items in the display list"
        );

        // Verify the first image (http://example.com/image.png) has been decoded
        if let DisplayItem::Image { src, decoded, .. } = image_items[0] {
            assert_eq!(src, "http://example.com/image.png");
            let decoded_img = decoded
                .as_ref()
                .expect("First image should have a decoded DecodedImage");
            assert_eq!(decoded_img.width, 2);
            assert_eq!(decoded_img.height, 2);
            assert_eq!(&decoded_img.rgba[0..4], &[255, 0, 0, 255]); // Red
        } else {
            panic!("Expected DisplayItem::Image");
        }

        // Verify the second image (data: URI) has been decoded
        if let DisplayItem::Image { src, decoded, .. } = image_items[1] {
            assert_eq!(src, &data_uri);
            let decoded_img = decoded
                .as_ref()
                .expect("Second image should have a decoded DecodedImage");
            assert_eq!(decoded_img.width, 2);
            assert_eq!(decoded_img.height, 2);
            assert_eq!(&decoded_img.rgba[0..4], &[255, 0, 0, 255]); // Red
        } else {
            panic!("Expected DisplayItem::Image");
        }
    }
}
