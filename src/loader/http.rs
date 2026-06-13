use crate::loader::{HttpMethod, LoadError, ResourceLoader};
use crate::url::Url;
use std::io::Read;
use std::sync::Mutex;

/// A minimal in-memory cookie representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookie {
    /// The name of the cookie.
    pub name: String,
    /// The value of the cookie.
    pub value: String,
    /// The domain scope of the cookie.
    pub domain: String,
    /// The path scope of the cookie.
    pub path: String,
}

// Global thread-safe cookie jar.
static COOKIE_JAR: Mutex<Vec<Cookie>> = Mutex::new(Vec::new());

/// Clears all stored cookies. Useful for testing.
#[allow(dead_code)]
pub fn clear_cookies() {
    let mut jar = COOKIE_JAR.lock().unwrap_or_else(|e| e.into_inner());
    jar.clear();
}

/// Returns a copy of the currently stored cookies.
#[cfg(test)]
pub fn get_cookies() -> Vec<Cookie> {
    let jar = COOKIE_JAR.lock().unwrap_or_else(|e| e.into_inner());
    jar.clone()
}

/// Helper to add a cookie to the global jar. If a cookie with the same name, domain,
/// and path already exists, it is overwritten.
fn add_cookie(new_cookie: Cookie) {
    let mut jar = COOKIE_JAR.lock().unwrap_or_else(|e| e.into_inner());
    jar.retain(|c| {
        !(c.name == new_cookie.name
            && c.domain.to_lowercase() == new_cookie.domain.to_lowercase()
            && c.path == new_cookie.path)
    });
    jar.push(new_cookie);
}

/// Helper to check if a cookie's domain matches the request host.
fn domain_match(cookie_domain: &str, request_host: &str) -> bool {
    let c_dom = cookie_domain.trim_start_matches('.').to_lowercase();
    let r_host = request_host.to_lowercase();
    if r_host == c_dom {
        return true;
    }
    if r_host.ends_with(&format!(".{}", c_dom)) {
        return true;
    }
    false
}

/// Helper to check if a cookie's path matches the request path.
fn path_match(cookie_path: &str, request_path: &str) -> bool {
    if cookie_path == "/" || cookie_path.is_empty() {
        return true;
    }
    if request_path == cookie_path {
        return true;
    }
    if request_path.starts_with(cookie_path) {
        if cookie_path.ends_with('/') {
            return true;
        }
        let next_char = request_path.chars().nth(cookie_path.chars().count());
        if next_char == Some('/') || next_char.is_none() {
            return true;
        }
    }
    false
}

/// Parses a `Set-Cookie` header value and returns a `Cookie` struct.
fn parse_set_cookie(header_val: &str, request_host: &str, request_path: &str) -> Option<Cookie> {
    let mut parts = header_val.split(';');
    let first_part = parts.next()?;
    let (name_part, value_part) = first_part.split_once('=')?;
    let name = name_part.trim().to_string();
    let value = value_part.trim().to_string();
    if name.is_empty() {
        return None;
    }

    let mut domain = request_host.to_string();

    // spec: default path is directory of request_path
    let mut path = "/".to_string();
    if let Some(last_slash) = request_path.rfind('/') {
        if last_slash > 0 {
            path = request_path[..last_slash].to_string();
        } else {
            path = "/".to_string();
        }
    }

    for part in parts {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            let key = k.trim().to_lowercase();
            let mut val = v.trim().to_string();
            if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                val = val[1..val.len() - 1].to_string();
            }
            if key == "domain" && !val.is_empty() {
                domain = val;
            } else if key == "path" && !val.is_empty() {
                path = val;
            }
        }
    }

    Some(Cookie {
        name,
        value,
        domain,
        path,
    })
}

/// Gets the merged `Cookie` header string for a given host and path.
fn get_cookie_header(host: &str, path: &str) -> Option<String> {
    let jar = COOKIE_JAR.lock().unwrap_or_else(|e| e.into_inner());
    let mut matching_cookies = Vec::new();
    for cookie in jar.iter() {
        if domain_match(&cookie.domain, host) && path_match(&cookie.path, path) {
            matching_cookies.push(format!("{}={}", cookie.name, cookie.value));
        }
    }
    if matching_cookies.is_empty() {
        None
    } else {
        Some(matching_cookies.join("; "))
    }
}

/// An HTTP(S) resource loader using the `ureq` crate.
pub struct HttpLoader;

impl HttpLoader {
    /// Performs an HTTP POST request sending the specified body and Content-Type header.
    /// Supports the cookie jar (Set-Cookie/Cookie).
    ///
    /// # Errors
    /// Returns `LoadError` if the request fails or scheme is unsupported.
    pub fn post(&self, url: &Url, body: &[u8], content_type: &str) -> Result<Vec<u8>, LoadError> {
        // // spec: support http and https schemes
        if url.scheme != "http" && url.scheme != "https" {
            return Err(LoadError::UnsupportedScheme);
        }

        let url_str = url.serialize();
        let mut req = ureq::post(url_str);

        // Add cookies
        if let Some(cookie_hdr) = get_cookie_header(url.host.as_deref().unwrap_or(""), &url.path) {
            req = req.header("Cookie", cookie_hdr);
        }

        // Add Content-Type header
        req = req.header("Content-Type", content_type);

        // Send request with body
        let response = req.send(body).map_err(|e| match e {
            ureq::Error::StatusCode(404) => LoadError::NotFound,
            _ => LoadError::Io(e.to_string()),
        })?;

        // Extract Set-Cookie headers
        for header_value in response.headers().get_all("set-cookie") {
            if let Some(cookie) = header_value.to_str().ok().and_then(|cookie_str| {
                parse_set_cookie(cookie_str, url.host.as_deref().unwrap_or(""), &url.path)
            }) {
                add_cookie(cookie);
            }
        }

        let mut res_body = Vec::new();
        response
            .into_body()
            .into_reader()
            .read_to_end(&mut res_body)
            .map_err(|e| LoadError::Io(e.to_string()))?;

        Ok(res_body)
    }
}

impl ResourceLoader for HttpLoader {
    fn load_request_hop(
        &self,
        url: &Url,
        method: HttpMethod,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<(crate::loader::RedirectMeta, crate::loader::LoaderResponse), LoadError> {
        if url.scheme != "http" && url.scheme != "https" {
            return Err(LoadError::UnsupportedScheme);
        }

        let url_str = url.serialize();
        let agent_config = ureq::Agent::config_builder().max_redirects(0).build();
        let agent = ureq::Agent::new_with_config(agent_config);

        let cookie_hdr = get_cookie_header(url.host.as_deref().unwrap_or(""), &url.path);

        let response_result = if method == HttpMethod::Post {
            let mut req = agent.post(&url_str);
            if let Some(c) = cookie_hdr {
                req = req.header("Cookie", c);
            }
            let ct = content_type.unwrap_or("application/x-www-form-urlencoded");
            req = req.header("Content-Type", ct);
            req.send(body)
        } else {
            let mut req = agent.get(&url_str);
            if let Some(c) = cookie_hdr {
                req = req.header("Cookie", c);
            }
            req.call()
        };

        let response = match response_result {
            Ok(resp) => resp,
            Err(e) => {
                if let ureq::Error::StatusCode(code) = e
                    && code == 404
                {
                    return Err(LoadError::NotFound);
                }
                return Err(LoadError::Io(e.to_string()));
            }
        };

        let status = response.status().into();
        let location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        for header_value in response.headers().get_all("set-cookie") {
            if let Some(cookie) = header_value.to_str().ok().and_then(|cookie_str| {
                parse_set_cookie(cookie_str, url.host.as_deref().unwrap_or(""), &url.path)
            }) {
                add_cookie(cookie);
            }
        }

        let transport_content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());

        let mut out_body = Vec::new();
        response
            .into_body()
            .into_reader()
            .read_to_end(&mut out_body)
            .map_err(|e| LoadError::Io(e.to_string()))?;

        let (content_type, charset) =
            crate::loader::sniff_response(&out_body, url, transport_content_type.as_deref());

        Ok((
            crate::loader::RedirectMeta { status, location },
            crate::loader::LoaderResponse {
                bytes: out_body,
                content_type,
                charset,
            },
        ))
    }

    fn load_request(
        &self,
        url: &Url,
        method: HttpMethod,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<crate::loader::LoaderResponse, LoadError> {
        let (resp, _final_url) = crate::loader::follow_redirects(url, |u| {
            self.load_request_hop(u, method, body, content_type)
        })?;
        Ok(resp)
    }

    fn load_rich(&self, url: &Url) -> Result<crate::loader::LoaderResponse, LoadError> {
        self.load_request(url, HttpMethod::Get, &[], None)
    }

    fn load(&self, url: &Url) -> Result<Vec<u8>, LoadError> {
        self.load_rich(url).map(|r| r.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_http_loader_unsupported_scheme() {
        let loader = HttpLoader;
        let url = Url::parse("file:///test.txt").unwrap();
        let result = loader.load(&url);
        assert_eq!(result, Err(LoadError::UnsupportedScheme));
    }

    #[test]
    #[ignore] // CI must pass without network
    fn test_http_loader_real_network() {
        let loader = HttpLoader;
        let url = Url::parse("http://example.com/").unwrap();
        let result = loader.load(&url).unwrap();
        assert!(!result.is_empty());
        assert!(String::from_utf8_lossy(&result).contains("Example Domain"));
    }

    /// Reads an entire HTTP request from a TcpStream.
    /// Sleeps a tiny bit to ensure ureq has fully flushed headers and body.
    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        thread::sleep(std::time::Duration::from_millis(20));
        let mut buffer = [0; 4096];
        let mut request_data = Vec::new();
        if let Ok(n) = stream.read(&mut buffer) {
            request_data.extend_from_slice(&buffer[..n]);
        }
        String::from_utf8_lossy(&request_data).to_string()
    }

    #[test]
    fn test_cookie_jar_store_and_send() {
        clear_cookies();

        // Bind to an arbitrary available local port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn local mock HTTP server
        thread::spawn(move || {
            // First Request: Expect GET, respond with Set-Cookie
            if let Ok((mut stream, _)) = listener.accept() {
                let _request_str = read_http_request(&mut stream);

                let response = "HTTP/1.1 200 OK\r\n\
                                Set-Cookie: session_id=abc123val; Path=/; Domain=127.0.0.1\r\n\
                                Content-Length: 12\r\n\r\n\
                                cookie-saved";
                stream.write_all(response.as_bytes()).unwrap();
            }

            // Second Request: Expect GET with Cookie header
            if let Ok((mut stream, _)) = listener.accept() {
                let request_str = read_http_request(&mut stream);

                if request_str
                    .to_lowercase()
                    .contains("cookie: session_id=abc123val")
                {
                    let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Length: 11\r\n\r\n\
                                    cookie-sent";
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\
                                    Content-Length: 13\r\n\r\n\
                                    missing-cookie";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        });

        let loader = HttpLoader;

        // 1. Initial request to trigger Set-Cookie
        let url1 = Url::parse(&format!("http://127.0.0.1:{}/set", port)).unwrap();
        let res1 = loader.load(&url1).unwrap();
        assert_eq!(String::from_utf8_lossy(&res1), "cookie-saved");

        // Verify stored cookie
        let cookies = get_cookies();
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session_id");
        assert_eq!(cookies[0].value, "abc123val");

        // 2. Subsequent request to verify Cookie header is sent back
        let url2 = Url::parse(&format!("http://127.0.0.1:{}/get", port)).unwrap();
        let res2 = loader.load(&url2).unwrap();
        assert_eq!(String::from_utf8_lossy(&res2), "cookie-sent");
    }

    #[test]
    fn test_post_request_body() {
        clear_cookies();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let request_str = read_http_request(&mut stream);

                // Verify request is POST, has correct content-type, and contains the body
                if request_str.starts_with("POST ")
                    && request_str
                        .to_lowercase()
                        .contains("content-type: application/x-www-form-urlencoded")
                    && request_str.contains("name=john&age=30")
                {
                    let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Length: 7\r\n\r\n\
                                    post-ok";
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\
                                    Content-Length: 9\r\n\r\n\
                                    post-fail";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        });

        let loader = HttpLoader;
        let url = Url::parse(&format!("http://127.0.0.1:{}/submit", port)).unwrap();
        let body = b"name=john&age=30";
        let res = loader
            .post(&url, body, "application/x-www-form-urlencoded")
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&res), "post-ok");
    }

    #[test]
    fn test_load_request_default_and_get() {
        // A simple custom ResourceLoader that implements only load()
        struct SimpleMockLoader;
        impl ResourceLoader for SimpleMockLoader {
            fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
                Ok(b"hello-world".to_vec())
            }
        }

        let loader = SimpleMockLoader;
        let url = Url::parse("http://example.com/").unwrap();

        // GET request via load_request should succeed and return sniffed LoaderResponse
        let res = loader
            .load_request(&url, HttpMethod::Get, &[], None)
            .unwrap();
        assert_eq!(res.bytes, b"hello-world");
        assert_eq!(res.content_type, "text/html");

        // Any non-GET request (like POST) via the default implementation should return UnsupportedScheme
        let res_post = loader.load_request(&url, HttpMethod::Post, &[], None);
        assert_eq!(res_post, Err(LoadError::UnsupportedScheme));
    }

    #[test]
    fn test_http_loader_load_request_post() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let request_str = read_http_request(&mut stream);

                // Verify request is POST, has correct content-type, and contains the body
                let is_post = request_str.starts_with("POST ");
                let has_ct = request_str
                    .to_lowercase()
                    .contains("content-type: application/custom-test");
                let has_body = request_str.contains("my-body-content");

                if is_post && has_ct && has_body {
                    let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: 9\r\n\r\n\
                                    custom-ok";
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\
                                    Content-Length: 9\r\n\r\n\
                                    post-fail";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        });

        let loader = HttpLoader;
        let url = Url::parse(&format!("http://127.0.0.1:{}/submit", port)).unwrap();
        let body = b"my-body-content";
        let res = loader
            .load_request(
                &url,
                HttpMethod::Post,
                body,
                Some("application/custom-test"),
            )
            .unwrap();
        assert_eq!(res.bytes, b"custom-ok");
        assert_eq!(res.content_type, "text/html");
    }
}
