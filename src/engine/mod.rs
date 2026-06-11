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
pub const UA_DEFAULT_CSS: &str = "\
html, body { background: #fff; background-color: #fff; color: #000; }\n\
body { margin: 8px; }\n\
div, p, h1, h2, h3, h4, h5, h6, ul, ol, li, section, header, footer, nav, article, figure, figcaption, blockquote, dl, dt, dd { display: block; }\n\
p { margin-top: 1em; margin-bottom: 1em; }\n\
h1 { margin-top: 0.67em; margin-bottom: 0.67em; font-weight: bold; }\n\
h2 { margin-top: 0.83em; margin-bottom: 0.83em; font-weight: bold; }\n\
h3 { margin-top: 1em; margin-bottom: 1em; font-weight: bold; }\n\
h4 { margin-top: 1.33em; margin-bottom: 1.33em; font-weight: bold; }\n\
h5 { margin-top: 1.67em; margin-bottom: 1.67em; font-weight: bold; }\n\
h6 { margin-top: 2.33em; margin-bottom: 2.33em; font-weight: bold; }\n\
figure, blockquote { margin: 1em 40px; }\n\
dl { margin: 1em 0; }\n\
dd { margin-left: 40px; }\n\
ul, ol { margin: 1em 0; padding-left: 40px; }\n\
a { color: #0000ee; text-decoration: underline; }\n\
b, strong { font-weight: bold; }\n\
i, em { font-style: italic; }\n\
th { font-weight: bold; text-align: center; }\n\
s, strike, del { text-decoration: line-through; }\n\
u, ins { text-decoration: underline; }\n\
mark { background-color: #ffff00; color: #000; }\n\
center { text-align: center; }\n\
address { display: block; font-style: italic; }\n\
head, style, script, meta, link, title { display: none; }\n\
button, input[type=\"submit\"], input[type=\"button\"], input[type=\"reset\"] {\n\
  display: inline-block;\n\
  padding: 1px 6px;\n\
  border: 2px outset #c0c0c0;\n\
  border-color: #c0c0c0;\n\
  background-color: #efefef;\n\
  color: #000;\n\
}\n\
input {\n\
  display: inline-block;\n\
  padding: 1px 2px;\n\
  border: 1px solid #767676;\n\
  border-color: #767676;\n\
  background-color: #fff;\n\
}\n\
input[type=\"text\"], input[type=\"search\"], input[type=\"email\"], input[type=\"url\"], input[type=\"tel\"], input[type=\"password\"], input[type=\"number\"] {\n\
  display: inline-block;\n\
  padding: 1px 2px;\n\
  border: 1px solid #767676;\n\
  border-color: #767676;\n\
  background-color: #fff;\n\
}\n\
";

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
    viewport_width: f32,
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
        {
            let mut chosen_url = src.to_string();
            let mut resolved_from_picture = false;

            // Check if <img> is a child of a <picture> element
            if let Some(parent_id) = dom.parent(n_id)
                && let Some(NodeData::Element {
                    name: parent_name, ..
                }) = dom.data(parent_id)
                && parent_name.eq_ignore_ascii_case("picture")
            {
                for &sibling_id in dom.children(parent_id) {
                    if sibling_id == n_id {
                        break;
                    }
                    if let Some(NodeData::Element { name: sib_name, .. }) = dom.data(sibling_id)
                        && sib_name.eq_ignore_ascii_case("source")
                    {
                        if let Some(source_type) = dom.get_attribute(sibling_id, "type") {
                            let mime = if let Some(part) = source_type.split(';').next() {
                                part.trim().to_ascii_lowercase()
                            } else {
                                String::new()
                            };
                            let is_supported = matches!(
                                mime.as_str(),
                                "image/png"
                                    | "image/jpeg"
                                    | "image/jpg"
                                    | "image/gif"
                                    | "image/bmp"
                                    | "image/webp"
                            );
                            if !is_supported {
                                continue;
                            }
                        }

                        if let Some(media) = dom.get_attribute(sibling_id, "media")
                            && !crate::css::media::media_matches(media, viewport_width)
                        {
                            continue;
                        }

                        if let Some(srcset) = dom.get_attribute(sibling_id, "srcset")
                            && !srcset.is_empty()
                        {
                            let candidates = crate::html::parse_srcset(srcset);
                            let effective_px = crate::html::resolve_sizes(
                                dom.get_attribute(sibling_id, "sizes"),
                                viewport_width as u32,
                            );
                            if let Some(c) =
                                crate::html::select_candidate(&candidates, 1.0, effective_px)
                            {
                                chosen_url = c.url.clone();
                                resolved_from_picture = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !resolved_from_picture
                && let Some(srcset) = dom.get_attribute(n_id, "srcset")
                && !srcset.is_empty()
            {
                let candidates = crate::html::parse_srcset(srcset);
                let effective_px = crate::html::resolve_sizes(
                    dom.get_attribute(n_id, "sizes"),
                    viewport_width as u32,
                );
                if let Some(c) = crate::html::select_candidate(&candidates, 1.0, effective_px) {
                    chosen_url = c.url.clone();
                }
            }

            if let Some(bytes) =
                load_image_safely_with_loader(loader, &chosen_url, Some(&effective_base))
                && let Some(decoded) = crate::image::decode_image(&bytes)
            {
                dom.add_image(src.to_string(), decoded);
            }
        }
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

    // 5. Parse accumulated stylesheet
    let stylesheet = crate::css::parser::parse_stylesheet(&css_accumulator);

    // Compute styles before running scripts so that inline scripts have access to them
    let pre_script_styles =
        crate::style::compute_styles_with_viewport(&dom, &stylesheet, viewport_width);

    // 4. Run inline scripts with computed styles
    dom = crate::script::run_inline_scripts(dom, &pre_script_styles);

    // 6. Compute styles again with viewport (since scripts might have mutated the DOM)
    let styles = crate::style::compute_styles_with_viewport(&dom, &stylesheet, viewport_width);

    // 7. Layout document
    let layout = crate::layout::layout_document(&dom, &styles, viewport_width);

    fetch_and_decode_images(&dom, base_url, loader, viewport_width);

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

    let response = match crate::loader::follow_redirects(&resolved_url, |url| {
        loader.load_request_hop(
            url,
            method,
            req.body.as_bytes(),
            req.content_type.as_deref(),
        )
    }) {
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

    // TODO(spec): after a redirect chain, relative URLs on the result page should resolve against the final hop URL, but follow_redirects does not surface it yet (needs a loader IF extension — future task).
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

/// Resolves a click on `clicked` into a navigated result page.
///
/// Returns `Some(Page)` when `clicked` is a submit button that owns a form
/// (`forms::submit_from_button` yields a `NavigationRequest`): the request is
/// dispatched through `navigate` against `base` and the rendered result page
/// is returned. Returns `None` when the click does not trigger a submission
/// (not a submit button, or no owning form). Never panics (I-6).
pub fn navigate_from_click(
    dom: &crate::dom::Dom,
    clicked: crate::infra::NodeId,
    values: &crate::forms::FormState,
    base: &Url,
    loader: &dyn ResourceLoader,
    viewport_width: f32,
) -> Option<Page> {
    let req = crate::forms::submit_from_button(dom, clicked, values)?;
    Some(navigate(&req, base, loader, viewport_width))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeData;
    struct DummyLoader;
    impl crate::loader::ResourceLoader for DummyLoader {
        fn load(&self, _url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
            Err(crate::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &crate::url::Url,
            _method: crate::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<crate::loader::LoaderResponse, crate::loader::LoadError> {
            Err(crate::loader::LoadError::NotFound)
        }
    }

    fn render_for_test(html: &str, css: &str, viewport_width: f32) -> Page {
        let combined = if css.is_empty() {
            html.to_string()
        } else {
            format!("{}<style>{}</style>", html, css)
        };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();
        render_page(&combined, &base_url, &DummyLoader, viewport_width)
    }

    fn render_to_canvas_for_test(
        html: &str,
        css: &str,
        width: u32,
        height: u32,
    ) -> crate::raster::Canvas {
        let combined = if css.is_empty() {
            html.to_string()
        } else {
            format!("{}<style>{}</style>", html, css)
        };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();
        render_page_to_canvas(&combined, &base_url, &DummyLoader, width, height)
    }

    fn render_html_for_test(html: &str, viewport_width: f32) -> Page {
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();
        render_page(html, &base_url, &DummyLoader, viewport_width)
    }

    fn render_html_to_canvas_for_test(
        html: &str,
        width: u32,
        height: u32,
    ) -> crate::raster::Canvas {
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();
        render_page_to_canvas(html, &base_url, &DummyLoader, width, height)
    }

    #[test]
    fn test_smoke_render_for_test() {
        let html = "<html><body><div></div></body></html>";
        let css = "div { width: 100px; height: 50px; }";
        let viewport_width = 800.0;

        let page = render_for_test(html, css, viewport_width);

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
    fn test_render_to_canvas_for_test() {
        let html = "<div></div>";
        let css = "html, body { background-color: transparent; } body { margin: 0; } div { background-color: #ff0000; width: 10px; height: 10px; }";
        let width = 20;
        let height = 20;

        let canvas = render_to_canvas_for_test(html, css, width, height);

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
    fn test_render_html_for_test() {
        let html = "<html><head><style>div { width: 100px; height: 50px; background-color: #ff0000; }</style></head><body><div></div></body></html>";
        let viewport_width = 800.0;

        let page = render_html_for_test(html, viewport_width);

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
    fn test_render_html_to_canvas_for_test() {
        let html = "<html><head><style>html, body { background-color: transparent; } body { margin: 0; } div { background-color: #ff0000; width: 10px; height: 10px; }</style></head><body><div></div></body></html>";
        let width = 20;
        let height = 20;

        let canvas = render_html_to_canvas_for_test(html, width, height);

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

        let page = render_html_for_test(html, viewport_width);

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
        let page = render_page(html, &base_url, &loader, 800.0);

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

        let page = render_page(html, &base_url, &loader, 800.0);

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
    fn test_render_page_script_get_computed_style() {
        let html = "
            <html>
              <head>
                <style>
                  #target { color: red; }
                </style>
              </head>
              <body>
                <div id=\"target\"></div>
                <script>
                  let el = document.getElementById('target');
                  let style = window.getComputedStyle(el);
                  let color = style.getPropertyValue('color');
                  if (color === 'rgb(255, 0, 0)') {
                      el.textContent = 'matched-color';
                  } else {
                      el.textContent = 'unmatched-color: ' + color;
                  }
                </script>
              </body>
            </html>
        ";

        let loader = MockLoader {
            responses: HashMap::new(),
        };
        let base_url = crate::url::Url::parse("https://example.com/").unwrap();

        let page = render_page(html, &base_url, &loader, 800.0);
        let doc = page.dom.document();

        let mut target_node_id = None;
        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { attrs, .. }) = page.dom.data(id)
                && attrs.iter().any(|(k, v)| k == "id" && v == "target")
            {
                target_node_id = Some(id);
                break;
            }
        }
        let target_id = target_node_id.expect("Should find element with id='target'");
        assert_eq!(page.dom.text_content(target_id), "matched-color");
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
        let page = render_html_for_test(html, 800.0);

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
    fn test_ua_default_stylesheet_figure() {
        let html = "<html><body><figure><img src=\"x\"><figcaption>Cap</figcaption></figure></body></html>";
        let page = render_html_for_test(html, 800.0);

        let doc = page.dom.document();
        let mut figure_style = None;
        let mut figcaption_style = None;

        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(id) {
                match name.as_str() {
                    "figure" => figure_style = page.styles.get(&id),
                    "figcaption" => figcaption_style = page.styles.get(&id),
                    _ => {}
                }
            }
        }

        let fig_s = figure_style.expect("figure should have styles");
        assert_eq!(
            fig_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );
        // figure should have margin-top/margin-bottom = 1em and margin-left/margin-right = 40px
        assert_eq!(
            fig_s.get("margin-top"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );
        assert_eq!(
            fig_s.get("margin-bottom"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );
        assert_eq!(
            fig_s.get("margin-left"),
            Some(&crate::css::values::CssValue::Length(
                40.0,
                crate::css::values::LengthUnit::Px
            ))
        );
        assert_eq!(
            fig_s.get("margin-right"),
            Some(&crate::css::values::CssValue::Length(
                40.0,
                crate::css::values::LengthUnit::Px
            ))
        );

        let figcap_s = figcaption_style.expect("figcaption should have styles");
        assert_eq!(
            figcap_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );
    }

    #[test]
    fn test_ua_default_stylesheet_blockquote_and_dl() {
        let html = "<html><body><blockquote>Q</blockquote><dl><dt>Term</dt><dd>Def</dd></dl></body></html>";
        let page = render_html_for_test(html, 800.0);

        let doc = page.dom.document();
        let mut blockquote_style = None;
        let mut dl_style = None;
        let mut dt_style = None;
        let mut dd_style = None;

        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(id) {
                match name.as_str() {
                    "blockquote" => blockquote_style = page.styles.get(&id),
                    "dl" => dl_style = page.styles.get(&id),
                    "dt" => dt_style = page.styles.get(&id),
                    "dd" => dd_style = page.styles.get(&id),
                    _ => {}
                }
            }
        }

        let bq_s = blockquote_style.expect("blockquote should have styles");
        assert_eq!(
            bq_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );
        assert_eq!(
            bq_s.get("margin-top"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );
        assert_eq!(
            bq_s.get("margin-bottom"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );
        assert_eq!(
            bq_s.get("margin-left"),
            Some(&crate::css::values::CssValue::Length(
                40.0,
                crate::css::values::LengthUnit::Px
            ))
        );
        assert_eq!(
            bq_s.get("margin-right"),
            Some(&crate::css::values::CssValue::Length(
                40.0,
                crate::css::values::LengthUnit::Px
            ))
        );

        let dl_s = dl_style.expect("dl should have styles");
        assert_eq!(
            dl_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );
        assert_eq!(
            dl_s.get("margin-top"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );
        assert_eq!(
            dl_s.get("margin-bottom"),
            Some(&crate::css::values::CssValue::Length(
                1.0,
                crate::css::values::LengthUnit::Em
            ))
        );

        let dt_s = dt_style.expect("dt should have styles");
        assert_eq!(
            dt_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );

        let dd_s = dd_style.expect("dd should have styles");
        assert_eq!(
            dd_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("block".to_string()))
        );
        assert_eq!(
            dd_s.get("margin-left"),
            Some(&crate::css::values::CssValue::Length(
                40.0,
                crate::css::values::LengthUnit::Px
            ))
        );
    }

    #[test]
    fn test_ua_default_stylesheet_th() {
        let html = "<html><body><table><tr><th>Header</th></tr></table></body></html>";
        let page = render_html_for_test(html, 800.0);

        let doc = page.dom.document();
        let mut th_style = None;

        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(id)
                && name.eq_ignore_ascii_case("th")
            {
                th_style = page.styles.get(&id);
                break;
            }
        }

        let th_s = th_style.expect("th should have styles");
        assert_eq!(
            th_s.get("font-weight"),
            Some(&crate::css::values::CssValue::Keyword("bold".to_string()))
        );
        assert_eq!(
            th_s.get("text-align"),
            Some(&crate::css::values::CssValue::Keyword("center".to_string()))
        );
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
    fn test_navigate_follows_redirects() {
        struct RedirectMockLoader;

        impl crate::loader::ResourceLoader for RedirectMockLoader {
            fn load(&self, _url: &Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }

            fn load_request_hop(
                &self,
                url: &Url,
                _method: crate::loader::HttpMethod,
                _body: &[u8],
                _content_type: Option<&str>,
            ) -> Result<
                (crate::loader::RedirectMeta, crate::loader::LoaderResponse),
                crate::loader::LoadError,
            > {
                if url.serialize() == "https://example.com/start" {
                    Ok((
                        crate::loader::RedirectMeta {
                            status: 302,
                            location: Some("/final".to_string()),
                        },
                        crate::loader::LoaderResponse {
                            bytes: b"Redirecting...".to_vec(),
                            content_type: "text/html".to_string(),
                            charset: Some("utf-8".to_string()),
                        },
                    ))
                } else if url.serialize() == "https://example.com/final" {
                    Ok((
                        crate::loader::RedirectMeta {
                            status: 200,
                            location: None,
                        },
                        crate::loader::LoaderResponse {
                            bytes: b"<html><body><h1>Redirected result</h1></body></html>".to_vec(),
                            content_type: "text/html".to_string(),
                            charset: Some("utf-8".to_string()),
                        },
                    ))
                } else {
                    Err(crate::loader::LoadError::NotFound)
                }
            }
        }

        let loader = RedirectMockLoader;
        let base_url = Url::parse("https://example.com/").unwrap();

        let get_request = crate::forms::NavigationRequest {
            url: "/start".to_string(),
            method: crate::forms::Method::Get,
            body: String::new(),
            content_type: None,
        };

        let page = navigate(&get_request, &base_url, &loader, 800.0);

        let mut found_text = false;
        let doc = page.dom.document();
        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Text(text)) = page.dom.data(id)
                && text.contains("Redirected result")
            {
                found_text = true;
            }
        }
        assert!(
            found_text,
            "Page after redirect should contain 'Redirected result' text"
        );
    }

    #[test]
    fn test_navigate_non_redirect_works() {
        struct NonRedirectMockLoader;

        impl crate::loader::ResourceLoader for NonRedirectMockLoader {
            fn load(&self, _url: &Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }

            fn load_request_hop(
                &self,
                _url: &Url,
                _method: crate::loader::HttpMethod,
                _body: &[u8],
                _content_type: Option<&str>,
            ) -> Result<
                (crate::loader::RedirectMeta, crate::loader::LoaderResponse),
                crate::loader::LoadError,
            > {
                Ok((
                    crate::loader::RedirectMeta {
                        status: 200,
                        location: None,
                    },
                    crate::loader::LoaderResponse {
                        bytes: b"<html><body><h1>Direct result</h1></body></html>".to_vec(),
                        content_type: "text/html".to_string(),
                        charset: Some("utf-8".to_string()),
                    },
                ))
            }
        }

        let loader = NonRedirectMockLoader;
        let base_url = Url::parse("https://example.com/").unwrap();

        let get_request = crate::forms::NavigationRequest {
            url: "/direct".to_string(),
            method: crate::forms::Method::Get,
            body: String::new(),
            content_type: None,
        };

        let page = navigate(&get_request, &base_url, &loader, 800.0);

        let mut found_text = false;
        let doc = page.dom.document();
        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Text(text)) = page.dom.data(id)
                && text.contains("Direct result")
            {
                found_text = true;
            }
        }
        assert!(
            found_text,
            "Page without redirect should contain 'Direct result' text"
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

    #[test]
    fn test_gui_render_path_hides_script_and_style() {
        use crate::paint::DisplayItem;

        let html = r#"
            <html>
                <head>
                    <title>My Page</title>
                    <style>
                        p { color: #ff0000; }
                    </style>
                    <script>
                        console.log("this should not be visible!");
                    </script>
                </head>
                <body>
                    <p>visible text</p>
                </body>
            </html>
        "#;

        // Render via the GUI-style render_for_test() path
        let page = render_for_test(html, "", 800.0);

        // 1. Assert style & script and head/title have display: none in computed styles
        let mut style_id = None;
        let mut script_id = None;
        let mut p_id = None;

        let doc = page.dom.document();
        for id in page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, .. }) = page.dom.data(id) {
                match name.as_str() {
                    "style" => style_id = Some(id),
                    "script" => script_id = Some(id),
                    "p" => p_id = Some(id),
                    _ => {}
                }
            }
        }

        let style_node = style_id.expect("should find style node");
        let script_node = script_id.expect("should find script node");
        let p_node = p_id.expect("should find p node");

        let style_s = page
            .styles
            .get(&style_node)
            .expect("style should have computed styles");
        assert_eq!(
            style_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("none".to_string()))
        );

        let script_s = page
            .styles
            .get(&script_node)
            .expect("script should have computed styles");
        assert_eq!(
            script_s.get("display"),
            Some(&crate::css::values::CssValue::Keyword("none".to_string()))
        );

        let p_s = page
            .styles
            .get(&p_node)
            .expect("p should have computed styles");
        if let Some(crate::css::values::CssValue::Color(c)) = p_s.get("color") {
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 0, 255));
        } else {
            panic!("Expected color red (#ff0000) for paragraph due to hoisted CSS");
        }

        // 2. Build display list and verify that no text from style or script is rendered,
        // but the paragraph text IS rendered.
        let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);
        let mut found_visible_text = false;
        let mut found_hidden_script_text = false;
        let mut found_hidden_style_text = false;

        for item in &display_list.0 {
            if let DisplayItem::Text { text, .. } = item {
                if text.contains("visible") {
                    found_visible_text = true;
                }
                if text.contains("console.log") {
                    found_hidden_script_text = true;
                }
                if text.contains("color: #ff0000") {
                    found_hidden_style_text = true;
                }
            }
        }

        assert!(
            found_visible_text,
            "Expected visible paragraph text in display list"
        );
        assert!(
            !found_hidden_script_text,
            "Script source text should not be visible in display list"
        );
        assert!(
            !found_hidden_style_text,
            "Style source text should not be visible in display list"
        );
    }

    #[test]
    fn test_remote_jpeg_fetch_decode_blit() {
        use crate::paint::DisplayItem;
        use std::collections::HashMap;

        // 1. JPEG base64 (corresponds to JPEG_BASE64_2 from image tests)
        let jpeg_base64 = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";
        let data_uri = format!("data:image/jpeg;base64,{}", jpeg_base64);

        // 2. Set up MockLoader
        let loader = MockLoader {
            responses: HashMap::new(),
        };

        // 3. Render HTML containing JPEG data URI
        let html = format!(
            r#"<html>
                <body>
                    <img id="img_jpeg" src="{}" style="width: 4px; height: 4px;">
                </body>
            </html>"#,
            data_uri
        );

        let base_url = crate::url::Url::parse("http://example.com/").unwrap();
        let page = render_page(&html, &base_url, &loader, 800.0);

        // 4. Verify JPEG is in the page DOM as a decoded image
        let image_items = page.dom.get_image(&data_uri);
        assert!(
            image_items.is_some(),
            "JPEG image should be present and decoded in the DOM"
        );
        let decoded_img = image_items.unwrap();
        assert_eq!(decoded_img.width, 1);
        assert_eq!(decoded_img.height, 1);
        assert_eq!(decoded_img.rgba.len(), 4);

        // Also build display list and verify it contains the Image item
        let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);
        let items = display_list.0;

        let image_items_list: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(
            image_items_list.len(),
            1,
            "Should have 1 image display item in the display list"
        );

        if let DisplayItem::Image { src, decoded, .. } = image_items_list[0] {
            assert_eq!(src, &data_uri);
            let decoded_img = decoded
                .as_ref()
                .expect("JPEG image should have a decoded DecodedImage in display list");
            assert_eq!(decoded_img.width, 1);
            assert_eq!(decoded_img.height, 1);
        } else {
            panic!("Expected DisplayItem::Image");
        }
    }

    #[test]
    fn test_navigate_from_click_complete_behavior() {
        use std::cell::RefCell;
        use std::collections::HashMap;

        // Custom recording loader
        struct RecordingLoader {
            requests: RefCell<Vec<(String, crate::loader::HttpMethod, Vec<u8>)>>,
            responses: HashMap<String, Vec<u8>>,
        }

        impl crate::loader::ResourceLoader for RecordingLoader {
            fn load(&self, _url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }

            fn load_request(
                &self,
                url: &crate::url::Url,
                method: crate::loader::HttpMethod,
                body: &[u8],
                _content_type: Option<&str>,
            ) -> Result<crate::loader::LoaderResponse, crate::loader::LoadError> {
                let serialized = url.serialize();
                self.requests
                    .borrow_mut()
                    .push((serialized.clone(), method, body.to_vec()));

                let bytes = self.responses.get(&serialized).cloned().unwrap_or_default();
                Ok(crate::loader::LoaderResponse {
                    bytes,
                    charset: Some("utf-8".to_string()),
                    content_type: "text/html".to_string(),
                })
            }
        }

        let base_url = Url::parse("https://example.com/").unwrap();

        // 1. GET form scenario
        let mut dom = Dom::new();
        let doc_root = dom.document();

        // Build a form: <form action="/search" method="get">
        let form = dom.create_node(NodeData::Element {
            name: "form".to_string(),
            attrs: vec![
                ("action".to_string(), "/search".to_string()),
                ("method".to_string(), "get".to_string()),
            ],
        });
        dom.append_child(doc_root, form);

        // Named input control: <input type="text" name="query" value="rust">
        let text_input = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("type".to_string(), "text".to_string()),
                ("name".to_string(), "query".to_string()),
                ("value".to_string(), "rust".to_string()),
            ],
        });
        dom.append_child(form, text_input);

        // Submit input: <input type="submit">
        let submit_input = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("type".to_string(), "submit".to_string())],
        });
        dom.append_child(form, submit_input);

        // Plain div inside the form
        let plain_div = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });
        dom.append_child(form, plain_div);

        // Button type=button inside the form
        let plain_button = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("type".to_string(), "button".to_string())],
        });
        dom.append_child(form, plain_button);

        // Prepare some mock responses
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/search?query=rust".to_string(),
            b"<html><body><h1>Search Results for Rust</h1></body></html>".to_vec(),
        );

        let loader = RecordingLoader {
            requests: RefCell::new(Vec::new()),
            responses,
        };

        let values = crate::forms::FormState::new();

        // Case 1: Clicking the submit button triggers a GET submission and returns Page
        let page_opt = navigate_from_click(&dom, submit_input, &values, &base_url, &loader, 800.0);
        assert!(
            page_opt.is_some(),
            "Clicking submit input should navigate and yield a Page"
        );

        // Assert loader recorded the correct GET request
        let recorded = loader.requests.borrow().clone();
        assert_eq!(recorded.len(), 1, "Exactly one request should be recorded");
        assert_eq!(recorded[0].0, "https://example.com/search?query=rust");
        assert_eq!(recorded[0].1, crate::loader::HttpMethod::Get);

        // Assert resulting page DOM contains the loaded content
        let navigated_page = page_opt.unwrap();
        let mut found_search_text = false;
        let nav_doc = navigated_page.dom.document();
        for id in navigated_page.dom.descendants(nav_doc) {
            if let Some(NodeData::Text(text)) = navigated_page.dom.data(id)
                && text.contains("Search Results for Rust")
            {
                found_search_text = true;
            }
        }
        assert!(
            found_search_text,
            "Navigated page should contain expected text"
        );

        // Clear requests log
        loader.requests.borrow_mut().clear();

        // Case 2: Clicking non-submit nodes returns None and records no request
        let div_result = navigate_from_click(&dom, plain_div, &values, &base_url, &loader, 800.0);
        assert!(
            div_result.is_none(),
            "Clicking plain div should not trigger navigation"
        );
        assert!(
            loader.requests.borrow().is_empty(),
            "No requests should be recorded for plain div click"
        );

        let button_result =
            navigate_from_click(&dom, plain_button, &values, &base_url, &loader, 800.0);
        assert!(
            button_result.is_none(),
            "Clicking button of type button should not trigger navigation"
        );
        assert!(
            loader.requests.borrow().is_empty(),
            "No requests should be recorded for plain button click"
        );

        // Case 3: A submit button with no owning form returns None
        let mut orphan_dom = Dom::new();
        let orphan_submit = orphan_dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("type".to_string(), "submit".to_string())],
        });
        // We do not append it to any form
        let orphan_result = navigate_from_click(
            &orphan_dom,
            orphan_submit,
            &values,
            &base_url,
            &loader,
            800.0,
        );
        assert!(
            orphan_result.is_none(),
            "Clicking orphan submit should not trigger navigation"
        );
        assert!(
            loader.requests.borrow().is_empty(),
            "No requests should be recorded for orphan submit click"
        );
    }

    #[test]
    fn test_srcset_honored() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate a valid PNG image
        let mut source_canvas = Canvas::new(2, 2);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        let png_bytes = crate::image::encode_png(&source_canvas);

        // 2. Set up MockLoader to return PNG bytes for hi.png, but NOT fallback.png
        let mut responses = HashMap::new();
        responses.insert("http://localhost/hi.png".to_string(), Ok(png_bytes));

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 3. Render HTML containing image with srcset (using absolute URLs to bypass local relative parsing bug)
        let html =
            r#"<img src="http://localhost/fallback.png" srcset="http://localhost/hi.png 1x">"#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 4. Assert dom.get_image("http://localhost/fallback.png") is Some with the expected dimensions (2, 2)
        let img_opt = page.dom.get_image("http://localhost/fallback.png");
        assert!(
            img_opt.is_some(),
            "Image should be cached under key fallback.png"
        );
        let decoded = img_opt.unwrap();
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
    }

    #[test]
    fn test_no_srcset_unchanged() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate a valid PNG image
        let mut source_canvas = Canvas::new(2, 2);
        source_canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = crate::image::encode_png(&source_canvas);

        // 2. Set up MockLoader to return PNG bytes for plain.png
        let mut responses = HashMap::new();
        responses.insert("http://localhost/plain.png".to_string(), Ok(png_bytes));

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 3. Render HTML containing plain image
        let html = r#"<img src="http://localhost/plain.png">"#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 4. Assert dom.get_image("http://localhost/plain.png") is Some with the expected dimensions (2, 2)
        let img_opt = page.dom.get_image("http://localhost/plain.png");
        assert!(
            img_opt.is_some(),
            "Image should be cached under key plain.png"
        );
        let decoded = img_opt.unwrap();
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
    }

    #[test]
    fn test_selection_correctness_sizes() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate a valid PNG image
        let mut source_canvas = Canvas::new(2, 2);
        source_canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = crate::image::encode_png(&source_canvas);

        // 2. Set up MockLoader to return PNG bytes for lo.png
        let mut responses = HashMap::new();
        responses.insert("http://localhost/lo.png".to_string(), Ok(png_bytes));

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 3. Render HTML containing srcset with lo.png (1x) and hi.png (2x)
        // With DPR 1.0 (hardcoded in fetch_and_decode_images), select_candidate should select lo.png (1x)
        let html = r#"<img src="http://localhost/x.png" srcset="http://localhost/lo.png 1x, http://localhost/hi.png 2x">"#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 4. Assert dom.get_image("http://localhost/x.png") is Some because lo.png was selected and found in the mock loader
        let img_opt = page.dom.get_image("http://localhost/x.png");
        assert!(img_opt.is_some(), "Image should be cached under key x.png");
        let decoded = img_opt.unwrap();
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
    }

    #[test]
    fn test_picture_source_srcset_honored() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate wide.png (4x4)
        let mut canvas_wide = Canvas::new(4, 4);
        canvas_wide.pixels[0] = 0xFFFF0000;
        let png_wide = crate::image::encode_png(&canvas_wide);

        // 2. Generate fallback.png (2x2)
        let mut canvas_fallback = Canvas::new(2, 2);
        canvas_fallback.pixels[0] = 0xFF00FF00;
        let png_fallback = crate::image::encode_png(&canvas_fallback);

        // 3. Set up loader
        let mut responses = HashMap::new();
        responses.insert("http://localhost/wide.png".to_string(), Ok(png_wide));
        responses.insert(
            "http://localhost/fallback.png".to_string(),
            Ok(png_fallback),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 4. Render HTML with <picture> wrapper
        let html = r#"
            <picture>
                <source srcset="http://localhost/wide.png">
                <img src="http://localhost/fallback.png">
            </picture>
        "#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 5. Assert cached image is wide.png (width == 4)
        let img_opt = page.dom.get_image("http://localhost/fallback.png");
        assert!(
            img_opt.is_some(),
            "Image should be cached under key fallback.png"
        );
        let decoded = img_opt.unwrap();
        assert_eq!(
            decoded.width, 4,
            "Should have loaded wide.png (width 4) instead of fallback.png (width 2)"
        );
    }

    #[test]
    fn test_picture_unsupported_type_fallback() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate wide.png (4x4)
        let mut canvas_wide = Canvas::new(4, 4);
        canvas_wide.pixels[0] = 0xFFFF0000;
        let png_wide = crate::image::encode_png(&canvas_wide);

        // 2. Generate fallback.png (2x2)
        let mut canvas_fallback = Canvas::new(2, 2);
        canvas_fallback.pixels[0] = 0xFF00FF00;
        let png_fallback = crate::image::encode_png(&canvas_fallback);

        // 3. Set up loader
        let mut responses = HashMap::new();
        responses.insert("http://localhost/wide.png".to_string(), Ok(png_wide));
        responses.insert(
            "http://localhost/fallback.png".to_string(),
            Ok(png_fallback),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 4. Render HTML with unsupported type="image/avif"
        let html = r#"
            <picture>
                <source srcset="http://localhost/wide.png" type="image/avif">
                <img src="http://localhost/fallback.png">
            </picture>
        "#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 5. Assert cached image is fallback.png (width == 2)
        let img_opt = page.dom.get_image("http://localhost/fallback.png");
        assert!(
            img_opt.is_some(),
            "Image should be cached under key fallback.png"
        );
        let decoded = img_opt.unwrap();
        assert_eq!(
            decoded.width, 2,
            "Should have fallen back to fallback.png (width 2) since image/avif is unsupported"
        );
    }

    #[test]
    fn test_picture_bare_img_fallback() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate fallback.png (2x2)
        let mut canvas_fallback = Canvas::new(2, 2);
        canvas_fallback.pixels[0] = 0xFF00FF00;
        let png_fallback = crate::image::encode_png(&canvas_fallback);

        // 2. Set up loader
        let mut responses = HashMap::new();
        responses.insert(
            "http://localhost/fallback.png".to_string(),
            Ok(png_fallback),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 3. Render bare HTML img with no picture wrapper
        let html = r#"<img src="http://localhost/fallback.png">"#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 4. Assert cached image is fallback.png (width == 2)
        let img_opt = page.dom.get_image("http://localhost/fallback.png");
        assert!(
            img_opt.is_some(),
            "Image should be cached under key fallback.png"
        );
        let decoded = img_opt.unwrap();
        assert_eq!(decoded.width, 2);
    }

    #[test]
    fn test_picture_multiple_sources() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate wide.png (4x4) and narrow.png (3x3) and fallback.png (2x2)
        let canvas_wide = Canvas::new(4, 4);
        let png_wide = crate::image::encode_png(&canvas_wide);

        let canvas_narrow = Canvas::new(3, 3);
        let png_narrow = crate::image::encode_png(&canvas_narrow);

        let canvas_fallback = Canvas::new(2, 2);
        let png_fallback = crate::image::encode_png(&canvas_fallback);

        // 2. Set up loader
        let mut responses = HashMap::new();
        responses.insert("http://localhost/wide.png".to_string(), Ok(png_wide));
        responses.insert("http://localhost/narrow.png".to_string(), Ok(png_narrow));
        responses.insert(
            "http://localhost/fallback.png".to_string(),
            Ok(png_fallback),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 3. First source has unsupported type, second has supported type, fallback img has fallback.png
        let html = r#"
            <picture>
                <source srcset="http://localhost/wide.png" type="image/avif">
                <source srcset="http://localhost/narrow.png" type="image/png">
                <img src="http://localhost/fallback.png">
            </picture>
        "#;
        let page = render_page(html, &base_url, &loader, 800.0);

        // 4. Assert cached image is narrow.png (width == 3)
        let img_opt = page.dom.get_image("http://localhost/fallback.png");
        assert!(img_opt.is_some());
        let decoded = img_opt.unwrap();
        assert_eq!(
            decoded.width, 3,
            "Should have loaded narrow.png (width 3) as the first supported source"
        );
    }

    #[test]
    fn test_picture_source_media_matching() {
        use crate::raster::Canvas;
        use std::collections::HashMap;

        // 1. Generate wide.png (4x4), narrow.png (3x3), fallback.png (2x2)
        let canvas_wide = Canvas::new(4, 4);
        let png_wide = crate::image::encode_png(&canvas_wide);

        let canvas_narrow = Canvas::new(3, 3);
        let png_narrow = crate::image::encode_png(&canvas_narrow);

        let canvas_fallback = Canvas::new(2, 2);
        let png_fallback = crate::image::encode_png(&canvas_fallback);

        // 2. Set up loader
        let mut responses = HashMap::new();
        responses.insert("http://localhost/wide.png".to_string(), Ok(png_wide));
        responses.insert("http://localhost/narrow.png".to_string(), Ok(png_narrow));
        responses.insert(
            "http://localhost/fallback.png".to_string(),
            Ok(png_fallback),
        );

        let loader = MockLoader { responses };
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();

        // 3. Define the HTML with responsive sources using media queries
        let html = r#"
            <picture>
                <source srcset="http://localhost/wide.png" media="(min-width: 600px)">
                <source srcset="http://localhost/narrow.png" media="(max-width: 599px)">
                <img src="http://localhost/fallback.png">
            </picture>
        "#;

        // Case A: viewport width = 800.0 (matches min-width: 600px)
        let page_wide = render_page(html, &base_url, &loader, 800.0);
        let img_wide_opt = page_wide.dom.get_image("http://localhost/fallback.png");
        assert!(img_wide_opt.is_some());
        assert_eq!(
            img_wide_opt.unwrap().width,
            4,
            "Should have loaded wide.png at viewport width 800.0"
        );

        // Case B: viewport width = 500.0 (matches max-width: 599px)
        let page_narrow = render_page(html, &base_url, &loader, 500.0);
        let img_narrow_opt = page_narrow.dom.get_image("http://localhost/fallback.png");
        assert!(img_narrow_opt.is_some());
        assert_eq!(
            img_narrow_opt.unwrap().width,
            3,
            "Should have loaded narrow.png at viewport width 500.0"
        );
    }
}
