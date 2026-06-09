//! WHATWG URL Standard (basic) implementation.
//!
// spec: <https://url.spec.whatwg.org/>

mod encoding;

pub use encoding::{PercentEncodeSet, encode_query, parse_query, percent_decode, percent_encode};

/// A WHATWG URL.
///
// spec: <https://url.spec.whatwg.org/#url-class>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Url {
    pub scheme: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

/// Errors that can occur during URL parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UrlParseError {
    InvalidScheme,
    InvalidHost,
    InvalidPort,
    InvalidPath,
    MissingBase,
    // TODO(spec): Add more granular errors as per WHATWG URL validation errors
    ValidationError,
}

impl Url {
    /// Parses an absolute URL.
    // spec: <https://url.spec.whatwg.org/#basic-url-parser>
    pub fn parse(input: &str) -> Result<Self, UrlParseError> {
        Self::parse_internal(input, None)
    }

    /// Parses a URL against a base URL.
    // spec: <https://url.spec.whatwg.org/#basic-url-parser>
    pub fn parse_with_base(input: &str, base: &Url) -> Result<Self, UrlParseError> {
        Self::parse_internal(input, Some(base))
    }

    fn parse_internal(input: &str, base: Option<&Url>) -> Result<Self, UrlParseError> {
        // // spec: https://url.spec.whatwg.org/#basic-url-parser
        let mut input = input.trim_matches(|c: char| c <= '\u{0020}').to_string();
        input.retain(|c| c != '\t' && c != '\n' && c != '\r');

        let mut url = Url {
            scheme: String::new(),
            host: None,
            port: None,
            path: String::new(),
            query: None,
            fragment: None,
        };
        let mut path_segments: Vec<String> = Vec::new();

        let mut state = State::SchemeStart;
        let mut buffer = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut pointer = 0;
        let mut authority_pending_pointer = 0;

        while pointer <= chars.len() {
            let c = chars.get(pointer).cloned();

            match state {
                State::SchemeStart => {
                    if let Some(c) = c {
                        if c.is_ascii_alphabetic() {
                            buffer.push(c.to_ascii_lowercase());
                            state = State::Scheme;
                        } else {
                            state = State::NoScheme;
                            continue;
                        }
                    } else {
                        return Err(UrlParseError::ValidationError);
                    }
                }
                State::Scheme => {
                    if let Some(c) = c {
                        if c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.' {
                            buffer.push(c.to_ascii_lowercase());
                        } else if c == ':' {
                            url.scheme = buffer.clone();
                            buffer.clear();
                            if url.scheme == "file" {
                                state = State::File;
                            } else if is_special(&url.scheme) {
                                state = State::SpecialRelativeOrAuthority;
                            } else {
                                state = State::PathStart;
                            }
                        } else {
                            state = State::NoScheme;
                            buffer.clear();
                            pointer = 0;
                            continue;
                        }
                    } else {
                        return Err(UrlParseError::ValidationError);
                    }
                }
                State::NoScheme => {
                    if let Some(base) = base {
                        if let Some(c) = c {
                            if c == '#' {
                                url = base.clone();
                                path_segments = url
                                    .path
                                    .split('/')
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect();
                                url.fragment = Some(String::new());
                                state = State::Fragment;
                            } else if c == '?' {
                                url = base.clone();
                                path_segments = url
                                    .path
                                    .split('/')
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect();
                                url.query = Some(String::new());
                                state = State::Query;
                            } else {
                                state = State::Relative;
                                continue;
                            }
                        } else {
                            url = base.clone();
                            path_segments = url
                                .path
                                .split('/')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string())
                                .collect();
                        }
                    } else {
                        return Err(UrlParseError::MissingBase);
                    }
                }
                State::SpecialRelativeOrAuthority => {
                    if let Some(c) = c {
                        if c == '/' && chars.get(pointer + 1) == Some(&'/') {
                            state = State::SpecialAuthorityIgnoreSlashes;
                            pointer += 1;
                        } else {
                            state = State::Relative;
                            continue;
                        }
                    } else {
                        state = State::Relative;
                        continue;
                    }
                }
                State::SpecialAuthorityIgnoreSlashes => {
                    authority_pending_pointer = pointer;
                    if let Some(c) = c {
                        if c == '/' || c == '\\' {
                            // ignore
                        } else {
                            state = State::Authority;
                            continue;
                        }
                    } else {
                        state = State::Authority;
                        continue;
                    }
                }
                State::Authority => {
                    if let Some(c) = c {
                        if c == '@' {
                            buffer.clear();
                            state = State::Host;
                        } else if c == '/' || c == '\\' || c == '?' || c == '#' {
                            pointer = authority_pending_pointer;
                            state = State::Host;
                            continue;
                        }
                    } else {
                        pointer = authority_pending_pointer;
                        state = State::Host;
                        continue;
                    }
                }
                State::Host => {
                    // // spec: https://url.spec.whatwg.org/#host-state
                    // TODO(spec): IDNA and full host parsing
                    if let Some(c) = c {
                        if c == ':' {
                            url.host = Some(buffer.clone());
                            buffer.clear();
                            state = State::Port;
                        } else if c == '/' || c == '\\' || c == '?' || c == '#' {
                            url.host = Some(buffer.clone());
                            buffer.clear();
                            state = State::PathStart;
                            continue;
                        } else {
                            buffer.push(c);
                        }
                    } else {
                        url.host = Some(buffer.clone());
                        buffer.clear();
                        state = State::PathStart;
                        continue;
                    }
                }
                State::Port => {
                    if let Some(c) = c {
                        if c.is_ascii_digit() {
                            buffer.push(c);
                        } else if c == '/' || c == '\\' || c == '?' || c == '#' {
                            if !buffer.is_empty() {
                                url.port = buffer.parse().ok();
                            }
                            buffer.clear();
                            state = State::PathStart;
                            continue;
                        } else {
                            return Err(UrlParseError::InvalidPort);
                        }
                    } else {
                        if !buffer.is_empty() {
                            url.port = buffer.parse().ok();
                        }
                        buffer.clear();
                        state = State::PathStart;
                        continue;
                    }
                }
                State::Relative => {
                    // // spec: https://url.spec.whatwg.org/#relative-state
                    let base = base.ok_or(UrlParseError::MissingBase)?;
                    url.scheme = base.scheme.clone();
                    if let Some(c) = c {
                        if c == '/' || (is_special(&url.scheme) && c == '\\') {
                            state = State::RelativeSlash;
                        } else {
                            url.host = base.host.clone();
                            url.port = base.port;
                            path_segments = base
                                .path
                                .split('/')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string())
                                .collect();
                            url.query = base.query.clone();
                            if c == '?' {
                                url.query = Some(String::new());
                                state = State::Query;
                            } else if c == '#' {
                                url.fragment = Some(String::new());
                                state = State::Fragment;
                            } else {
                                url.query = None;
                                if !path_segments.is_empty() {
                                    path_segments.pop();
                                }
                                state = State::Path;
                                continue;
                            }
                        }
                    } else {
                        url.host = base.host.clone();
                        url.port = base.port;
                        path_segments = base
                            .path
                            .split('/')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                        url.query = base.query.clone();
                        url.fragment = base.fragment.clone();
                    }
                }
                State::RelativeSlash => {
                    // // spec: https://url.spec.whatwg.org/#relative-slash-state
                    if is_special(&url.scheme) && (c == Some('/') || c == Some('\\')) {
                        state = State::SpecialAuthorityIgnoreSlashes;
                    } else if c == Some('/') {
                        state = State::Authority;
                    } else {
                        let base = base.ok_or(UrlParseError::MissingBase)?;
                        url.host = base.host.clone();
                        url.port = base.port;
                        state = State::Path;
                        continue;
                    }
                }
                State::PathStart => {
                    if is_special(&url.scheme) {
                        if let Some(c) = c {
                            if c == '/' || c == '\\' {
                                state = State::Path;
                            } else {
                                state = State::Path;
                                continue;
                            }
                        } else {
                            state = State::Path;
                        }
                    } else {
                        state = State::Path;
                        continue;
                    }
                }
                State::Path => {
                    // // spec: https://url.spec.whatwg.org/#path-state
                    if let Some(c) = c {
                        if c == '/'
                            || (is_special(&url.scheme) && c == '\\')
                            || c == '?'
                            || c == '#'
                        {
                            let decoded = percent_decode(&buffer);
                            if decoded == b".." {
                                shorten_path(&url.scheme, &mut path_segments);
                                if c != '/' && !(is_special(&url.scheme) && c == '\\') {
                                    path_segments.push(String::new());
                                }
                            } else if decoded == b"." {
                                if c != '/' && !(is_special(&url.scheme) && c == '\\') {
                                    path_segments.push(String::new());
                                }
                            } else {
                                path_segments.push(buffer.clone());
                            }
                            buffer.clear();
                            if c == '?' {
                                url.query = Some(String::new());
                                state = State::Query;
                            } else if c == '#' {
                                url.fragment = Some(String::new());
                                state = State::Fragment;
                            }
                        } else {
                            let mut char_buf = [0; 4];
                            buffer.push_str(&percent_encode(
                                c.encode_utf8(&mut char_buf),
                                PercentEncodeSet::Path,
                            ));
                        }
                    } else {
                        let decoded = percent_decode(&buffer);
                        if decoded == b".." {
                            shorten_path(&url.scheme, &mut path_segments);
                        } else if decoded == b"." {
                            // ignore
                        } else {
                            path_segments.push(buffer.clone());
                        }
                        buffer.clear();
                    }
                }
                State::Query => {
                    if let Some(c) = c {
                        if c == '#' {
                            url.query = Some(buffer.clone());
                            buffer.clear();
                            state = State::Fragment;
                        } else {
                            let mut char_buf = [0; 4];
                            buffer.push_str(&percent_encode(
                                c.encode_utf8(&mut char_buf),
                                PercentEncodeSet::Query,
                            ));
                        }
                    } else {
                        url.query = Some(buffer.clone());
                        buffer.clear();
                    }
                }
                State::Fragment => {
                    if let Some(c) = c {
                        let mut char_buf = [0; 4];
                        buffer.push_str(&percent_encode(
                            c.encode_utf8(&mut char_buf),
                            PercentEncodeSet::Fragment,
                        ));
                    } else {
                        url.fragment = Some(buffer.clone());
                        buffer.clear();
                    }
                }
                State::File => {
                    // // spec: https://url.spec.whatwg.org/#file-state
                    url.scheme = "file".to_string();
                    url.host = Some(String::new());
                    if let Some(c) = c {
                        if c == '/' || c == '\\' {
                            state = State::FileSlash;
                        } else {
                            state = State::Relative;
                            continue;
                        }
                    } else {
                        state = State::Relative;
                        continue;
                    }
                }
                State::FileSlash => {
                    // // spec: https://url.spec.whatwg.org/#file-slash-state
                    if let Some(c) = c {
                        if c == '/' || c == '\\' {
                            state = State::FileHost;
                        } else {
                            state = State::Path;
                            continue;
                        }
                    } else {
                        state = State::Path;
                        continue;
                    }
                }
                State::FileHost => {
                    // // spec: https://url.spec.whatwg.org/#file-host-state
                    if let Some(c) = c {
                        if c == '/' || c == '\\' || c == '?' || c == '#' {
                            if buffer == "localhost" {
                                url.host = Some(String::new());
                            } else {
                                url.host = Some(buffer.clone());
                            }
                            buffer.clear();
                            state = State::PathStart;
                            continue;
                        } else {
                            buffer.push(c);
                        }
                    } else {
                        if buffer == "localhost" {
                            url.host = Some(String::new());
                        } else {
                            url.host = Some(buffer.clone());
                        }
                        buffer.clear();
                        state = State::PathStart;
                        continue;
                    }
                }
            }
            pointer += 1;
        }

        // Finalize path
        url.path = "/".to_string() + &path_segments.join("/");

        // Handle default ports
        if let Some(port) = url.port
            && is_default_port(&url.scheme, port)
        {
            url.port = None;
        }

        Ok(url)
    }

    /// Serializes the URL to a string.
    // spec: <https://url.spec.whatwg.org/#url-serializing>
    pub fn serialize(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.scheme);
        output.push_str("://");
        if let Some(host) = &self.host {
            output.push_str(host);
        }
        if let Some(port) = self.port {
            output.push(':');
            output.push_str(&port.to_string());
        }
        output.push_str(&self.path);
        if let Some(query) = &self.query {
            output.push('?');
            output.push_str(query);
        }
        if let Some(fragment) = &self.fragment {
            output.push('#');
            output.push_str(fragment);
        }
        output
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    SchemeStart,
    Scheme,
    NoScheme,
    SpecialRelativeOrAuthority,
    SpecialAuthorityIgnoreSlashes,
    Authority,
    Host,
    Port,
    File,
    FileSlash,
    FileHost,
    PathStart,
    Path,
    Relative,
    RelativeSlash,
    Query,
    Fragment,
}

fn is_special(scheme: &str) -> bool {
    matches!(scheme, "ftp" | "file" | "http" | "https" | "ws" | "wss")
}

fn is_default_port(scheme: &str, port: u16) -> bool {
    match scheme {
        "ftp" => port == 21,
        "http" => port == 80,
        "https" => port == 443,
        "ws" => port == 80,
        "wss" => port == 443,
        _ => false,
    }
}

fn shorten_path(scheme: &str, path: &mut Vec<String>) {
    // // spec: https://url.spec.whatwg.org/#shorten-a-urls-path
    if path.is_empty() {
        return;
    }
    if scheme == "file" && path.len() == 1 && is_windows_drive_letter(&path[0]) {
        return;
    }
    path.pop();
}

fn is_windows_drive_letter(s: &str) -> bool {
    s.len() == 2
        && s.chars().next().unwrap_or('\0').is_ascii_alphabetic()
        && (s.ends_with(':') || s.ends_with('|'))
}

/// Resolves a reference URL against a base URL according to RFC 3986 reference resolution.
// spec: <https://tools.ietf.org/html/rfc3986#section-5.2>
pub fn resolve(base: &Url, rel: &str) -> Option<Url> {
    Url::parse_with_base(rel, base).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_absolute_basic() {
        let url = Url::parse("https://example.com:8080/a/b?q=1#f").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, Some("example.com".to_string()));
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/a/b".to_string());
        assert_eq!(url.query, Some("q=1".to_string()));
        assert_eq!(url.fragment, Some("f".to_string()));
    }

    #[test]
    fn test_serialize_basic() {
        let input = "https://example.com:8080/a/b?q=1#f";
        let url = Url::parse(input).unwrap();
        assert_eq!(url.serialize(), input);
    }

    #[test]
    fn test_parse_relative() {
        let base = Url::parse("https://example.com/a/b").unwrap();

        let url = Url::parse_with_base("../c", &base).unwrap();
        assert_eq!(url.serialize(), "https://example.com/c");

        let url = Url::parse_with_base("/abs", &base).unwrap();
        assert_eq!(url.serialize(), "https://example.com/abs");

        let url = Url::parse_with_base("?x", &base).unwrap();
        assert_eq!(url.serialize(), "https://example.com/a/b?x");

        let url = Url::parse_with_base("#y", &base).unwrap();
        assert_eq!(url.serialize(), "https://example.com/a/b#y");
    }

    #[test]
    fn test_default_ports() {
        assert_eq!(Url::parse("http://example.com:80/").unwrap().port, None);
        assert_eq!(Url::parse("https://example.com:443/").unwrap().port, None);
        assert_eq!(
            Url::parse("https://example.com:8443/").unwrap().port,
            Some(8443)
        );
    }

    #[test]
    fn test_file_scheme() {
        let url = Url::parse("file:///tmp/test").unwrap();
        assert_eq!(url.scheme, "file");
        assert_eq!(url.host, Some("".to_string()));
        assert_eq!(url.path, "/tmp/test");
        assert_eq!(url.serialize(), "file:///tmp/test");
    }

    #[test]
    fn test_percent_encoding_during_parse() {
        let url = Url::parse("https://example.com/a b?q=#f").unwrap();
        assert_eq!(url.path, "/a%20b");
        assert_eq!(url.query, Some("q=".to_string())); // space in query is encoded but here q= is followed by nothing or space? 
        // Wait, "a b" path, then "?" starts query.

        let url2 = Url::parse("https://example.com/path?query space#frag space").unwrap();
        assert_eq!(url2.path, "/path");
        assert_eq!(url2.query, Some("query%20space".to_string()));
        assert_eq!(url2.fragment, Some("frag%20space".to_string()));
    }

    #[test]
    fn test_dot_segment_normalization_with_percent_encoding() {
        let url = Url::parse("https://example.com/a/%2e%2e/b").unwrap();
        assert_eq!(url.path, "/b");

        let url2 = Url::parse("https://example.com/a/./%2e/b").unwrap();
        assert_eq!(url2.path, "/a/b");
    }

    #[test]
    fn test_resolve() {
        let base = Url::parse("https://a.com/x/y").unwrap();

        // Absolute stays absolute
        assert_eq!(
            resolve(&base, "https://b.com/foo").unwrap().serialize(),
            "https://b.com/foo"
        );

        // Protocol-relative //host
        assert_eq!(
            resolve(&base, "//cdn/p").unwrap().serialize(),
            "https://cdn/p"
        );

        // Absolute path /p
        assert_eq!(resolve(&base, "/p").unwrap().serialize(), "https://a.com/p");

        // Relative path with ../ and ./ removal
        assert_eq!(
            resolve(&base, "../z").unwrap().serialize(),
            "https://a.com/z"
        );

        assert_eq!(
            resolve(&base, "./z").unwrap().serialize(),
            "https://a.com/x/z"
        );

        // Query-only ?q
        assert_eq!(
            resolve(&base, "?q=1").unwrap().serialize(),
            "https://a.com/x/y?q=1"
        );

        // Fragment-only #f
        assert_eq!(
            resolve(&base, "#f").unwrap().serialize(),
            "https://a.com/x/y#f"
        );

        // Malformed returns None (no panic)
        assert!(resolve(&base, "https://foo:abc/").is_none());
    }
}
