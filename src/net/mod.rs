//! Network: HTTP GET and POST request sending using `ureq`.
//!
//! This module implements HTTP client request capabilities, enabling POST requests with a body
//! and content-type, in addition to supporting basic GET requests and cookie management.
//!
//! spec: S-88 / t0156

use crate::loader::{LoadError, LoaderResponse};
use crate::url::Url;
use std::io::Read;
use std::sync::Mutex;

/// HTTP Method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

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

// Global thread-safe cookie jar for this module.
static COOKIE_JAR: Mutex<Vec<Cookie>> = Mutex::new(Vec::new());

/// Clears all stored cookies. Useful for testing.
#[allow(dead_code)]
pub fn clear_cookies() {
    if let Ok(mut jar) = COOKIE_JAR.lock() {
        jar.clear();
    }
}

/// Returns a copy of the currently stored cookies.
#[cfg(test)]
pub fn get_cookies() -> Vec<Cookie> {
    if let Ok(jar) = COOKIE_JAR.lock() {
        jar.clone()
    } else {
        Vec::new()
    }
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

    // Default path is directory of request_path
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

/// Performs an HTTP GET or POST request.
/// Supports redirection, gzip decoding, and a cookie jar.
///
/// # Errors
/// Returns `LoadError` if the request fails or scheme is unsupported.
pub fn send_request(
    url: &Url,
    method: HttpMethod,
    body: &[u8],
    content_type: Option<&str>,
) -> Result<LoaderResponse, LoadError> {
    if url.scheme != "http" && url.scheme != "https" {
        return Err(LoadError::UnsupportedScheme);
    }

    let url_str = url.serialize();

    let response = match method {
        HttpMethod::Get => {
            let mut req = ureq::get(&url_str);
            if let Some(cookie_hdr) =
                get_cookie_header(url.host.as_deref().unwrap_or(""), &url.path)
            {
                req = req.header("Cookie", &cookie_hdr);
            }
            req.call().map_err(|e| match e {
                ureq::Error::StatusCode(404) => LoadError::NotFound,
                _ => LoadError::Io(e.to_string()),
            })?
        }
        HttpMethod::Post => {
            let mut req = ureq::post(&url_str);
            if let Some(cookie_hdr) =
                get_cookie_header(url.host.as_deref().unwrap_or(""), &url.path)
            {
                req = req.header("Cookie", &cookie_hdr);
            }
            if let Some(ct) = content_type {
                req = req.header("Content-Type", ct);
            }
            req.send(body).map_err(|e| match e {
                ureq::Error::StatusCode(404) => LoadError::NotFound,
                _ => LoadError::Io(e.to_string()),
            })?
        }
    };

    // Extract Set-Cookie headers
    for header_value in response.headers().get_all("set-cookie") {
        if let Ok(cookie_str) = header_value.to_str()
            && let Some(cookie) =
                parse_set_cookie(cookie_str, url.host.as_deref().unwrap_or(""), &url.path)
        {
            add_cookie(cookie);
        }
    }

    // Extract Content-Type header
    let transport_content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string());

    let mut res_body = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut res_body)
        .map_err(|e| LoadError::Io(e.to_string()))?;

    let (ct, cs) = crate::loader::sniff_response(&res_body, url, transport_content_type.as_deref());

    Ok(LoaderResponse {
        bytes: res_body,
        content_type: ct,
        charset: cs,
    })
}

/// Fetches multiple URLs concurrently with GET, honoring a maximum number of in-flight
/// requests (`max_concurrency`, clamped to at least 1). Results are returned in the SAME
/// order as `urls`. Each element is the individual `send_request` outcome for that URL, so a
/// single failed fetch does NOT abort the others.
pub fn fetch_all_concurrent(
    urls: &[Url],
    max_concurrency: usize,
) -> Vec<Result<LoaderResponse, LoadError>> {
    if urls.is_empty() {
        return Vec::new();
    }

    let concurrency = max_concurrency.max(1).min(urls.len());
    let urls_shared = std::sync::Arc::new(urls.to_vec());
    let next_index = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut results_vec = Vec::with_capacity(urls.len());
    for _ in 0..urls.len() {
        results_vec.push(None);
    }
    let results = std::sync::Arc::new(std::sync::Mutex::new(results_vec));

    let mut handles = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let next_index = std::sync::Arc::clone(&next_index);
        let urls_shared = std::sync::Arc::clone(&urls_shared);
        let results = std::sync::Arc::clone(&results);

        let handle = std::thread::spawn(move || {
            loop {
                let idx = next_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if idx >= urls_shared.len() {
                    break;
                }
                let url = &urls_shared[idx];
                let res = send_request(url, HttpMethod::Get, &[], None);

                match results.lock() {
                    Ok(mut guard) => {
                        if idx < guard.len() {
                            guard[idx] = Some(res);
                        }
                    }
                    Err(_) => {
                        // Poisoned lock, gracefully skip writing to avoid panic.
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to finish
    let mut any_join_failed = false;
    for handle in handles {
        if handle.join().is_err() {
            any_join_failed = true;
        }
    }

    // Extract the results vector safely without unwrap or expect
    let locked_results = match results.lock() {
        Ok(mut guard) => std::mem::take(&mut *guard),
        Err(err) => std::mem::take(&mut *err.into_inner()),
    };

    let mut final_results = Vec::with_capacity(urls.len());
    for opt in locked_results {
        match opt {
            Some(res) => final_results.push(res),
            None => {
                let err_msg = if any_join_failed {
                    "Thread join failed during concurrent fetch".to_string()
                } else {
                    "Fetch task was not completed successfully".to_string()
                };
                final_results.push(Err(LoadError::Io(err_msg)));
            }
        }
    }

    final_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_net_unsupported_scheme() {
        let url = Url::parse("file:///test.txt").unwrap();
        let result = send_request(&url, HttpMethod::Get, &[], None);
        assert_eq!(result, Err(LoadError::UnsupportedScheme));
    }

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
    fn test_net_post_request_body_and_content_type() {
        clear_cookies();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let request_str = read_http_request(&mut stream);

                if request_str.starts_with("POST ")
                    && request_str
                        .to_lowercase()
                        .contains("content-type: application/x-www-form-urlencoded")
                    && request_str.contains("query=hello&submit=true")
                {
                    let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Type: application/json; charset=utf-8\r\n\
                                    Content-Length: 15\r\n\r\n\
                                    post-responseok";
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\
                                    Content-Length: 9\r\n\r\n\
                                    post-fail";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        });

        let url = Url::parse(&format!("http://127.0.0.1:{}/post", port)).unwrap();
        let body = b"query=hello&submit=true";
        let res = send_request(
            &url,
            HttpMethod::Post,
            body,
            Some("application/x-www-form-urlencoded"),
        )
        .unwrap();

        assert_eq!(String::from_utf8_lossy(&res.bytes), "post-responseok");
        assert_eq!(res.content_type, "application/json");
        assert_eq!(res.charset, Some("utf-8".to_string()));
    }

    #[test]
    fn test_net_cookie_jar_store_and_send() {
        clear_cookies();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            // First Request: GET, respond with Set-Cookie
            if let Ok((mut stream, _)) = listener.accept() {
                let _request_str = read_http_request(&mut stream);

                let response = "HTTP/1.1 200 OK\r\n\
                                Set-Cookie: net_session=xyz789; Path=/; Domain=127.0.0.1\r\n\
                                Content-Length: 10\r\n\r\n\
                                cookie-set";
                stream.write_all(response.as_bytes()).unwrap();
            }

            // Second Request: POST, expect Cookie header
            if let Ok((mut stream, _)) = listener.accept() {
                let request_str = read_http_request(&mut stream);

                if request_str
                    .to_lowercase()
                    .contains("cookie: net_session=xyz789")
                {
                    let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Length: 11\r\n\r\n\
                                    cookie-echo";
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\
                                    Content-Length: 12\r\n\r\n\
                                    no-cookie-rx";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        });

        // 1. Initial request to set cookie
        let url1 = Url::parse(&format!("http://127.0.0.1:{}/set_cookie", port)).unwrap();
        let res1 = send_request(&url1, HttpMethod::Get, &[], None).unwrap();
        assert_eq!(String::from_utf8_lossy(&res1.bytes), "cookie-set");

        // Verify stored cookie
        let cookies = get_cookies();
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "net_session");
        assert_eq!(cookies[0].value, "xyz789");

        // 2. Subsequent POST request to verify Cookie header is sent
        let url2 = Url::parse(&format!("http://127.0.0.1:{}/post_cookie", port)).unwrap();
        let res2 = send_request(&url2, HttpMethod::Post, b"some body", Some("text/plain")).unwrap();
        assert_eq!(String::from_utf8_lossy(&res2.bytes), "cookie-echo");
    }

    #[test]
    fn test_fetch_all_concurrent_order() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            // Serve 3 requests
            for _ in 0..3 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let request_str = read_http_request(&mut stream);
                    let response = if request_str.contains("GET /a ") {
                        "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nbody-aa"
                    } else if request_str.contains("GET /b ") {
                        "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nbody-bb"
                    } else if request_str.contains("GET /c ") {
                        "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nbody-cc"
                    } else {
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 7\r\n\r\nbody-44"
                    };
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        });

        let url_a = Url::parse(&format!("http://127.0.0.1:{}/a", port)).unwrap();
        let url_b = Url::parse(&format!("http://127.0.0.1:{}/b", port)).unwrap();
        let url_c = Url::parse(&format!("http://127.0.0.1:{}/c", port)).unwrap();

        // Pass them in a mixed order and assert that order is preserved
        let urls = vec![url_c, url_a, url_b];
        let results = fetch_all_concurrent(&urls, 2);

        assert_eq!(results.len(), 3);

        let res_c = results[0].as_ref().unwrap();
        let res_a = results[1].as_ref().unwrap();
        let res_b = results[2].as_ref().unwrap();

        assert_eq!(String::from_utf8_lossy(&res_c.bytes), "body-cc");
        assert_eq!(String::from_utf8_lossy(&res_a.bytes), "body-aa");
        assert_eq!(String::from_utf8_lossy(&res_b.bytes), "body-bb");
    }

    #[test]
    fn test_fetch_all_concurrent_respects_failure_isolation() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            // Serve 2 requests (since one is bad scheme and won't hit server)
            for _ in 0..2 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let request_str = read_http_request(&mut stream);
                    let response = if request_str.contains("GET /a ") {
                        "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nbody-aa"
                    } else if request_str.contains("GET /b ") {
                        "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nbody-bb"
                    } else {
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 7\r\n\r\nbody-44"
                    };
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        });

        let url_a = Url::parse(&format!("http://127.0.0.1:{}/a", port)).unwrap();
        let url_bad = Url::parse("file:///x").unwrap();
        let url_b = Url::parse(&format!("http://127.0.0.1:{}/b", port)).unwrap();

        let urls = vec![url_a, url_bad, url_b];
        let results = fetch_all_concurrent(&urls, 3);

        assert_eq!(results.len(), 3);

        let res_a = results[0].as_ref().unwrap();
        assert_eq!(String::from_utf8_lossy(&res_a.bytes), "body-aa");

        assert_eq!(results[1], Err(LoadError::UnsupportedScheme));

        let res_b = results[2].as_ref().unwrap();
        assert_eq!(String::from_utf8_lossy(&res_b.bytes), "body-bb");
    }

    #[test]
    fn test_fetch_all_concurrent_empty() {
        let results = fetch_all_concurrent(&[], 4);
        assert!(results.is_empty());
    }
}
