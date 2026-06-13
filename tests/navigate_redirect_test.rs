use underrated::engine::navigate;
use underrated::forms::{NavigationRequest, Method};
use underrated::loader::{ResourceLoader, LoaderResponse, RedirectMeta, LoadError, HttpMethod};
use underrated::url::Url;

struct MockRedirectLoader;
impl ResourceLoader for MockRedirectLoader {
    fn load(&self, url: &Url) -> Result<Vec<u8>, LoadError> {
        self.load_rich(url).map(|r| r.bytes)
    }

    fn load_request_hop(
        &self,
        url: &Url,
        _method: HttpMethod,
        _body: &[u8],
        _content_type: Option<&str>,
    ) -> Result<(RedirectMeta, LoaderResponse), LoadError> {
        let u = url.serialize();
        if u == "https://google.com/" {
            Ok((
                RedirectMeta {
                    status: 301,
                    location: Some("https://www.google.com/".to_string()),
                },
                LoaderResponse {
                    bytes: b"".to_vec(),
                    content_type: "text/html".to_string(),
                    charset: None,
                },
            ))
        } else if u == "https://www.google.com/" {
            Ok((
                RedirectMeta {
                    status: 200,
                    location: None,
                },
                LoaderResponse {
                    bytes: b"<!DOCTYPE html><html><body>ok</body></html>".to_vec(),
                    content_type: "text/html".to_string(),
                    charset: None,
                },
            ))
        } else {
            Err(LoadError::NotFound)
        }
    }
}

#[test]
fn test_navigate_follows_redirect_and_updates_base() {
    let loader = MockRedirectLoader;
    let base = Url::parse("https://google.com/").unwrap();
    let req = NavigationRequest {
        url: "https://google.com/".to_string(),
        method: Method::Get,
        body: String::new(),
        content_type: None,
    };
    
    let page = navigate(&req, &base, &loader, 800.0);
    // document base should be the final URL
    
    assert_eq!(page.url.serialize(), "https://www.google.com/");
}
