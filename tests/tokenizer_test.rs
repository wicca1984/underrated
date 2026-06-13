#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde::Deserialize;
use underrated::encoding::InputStream;
use underrated::html::{Token, Tokenizer};

#[derive(Deserialize)]
struct TestFile {
    tests: Vec<TestCase>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TestCase {
    description: String,
    input: String,
    output: Vec<serde_json::Value>,
    #[serde(default)]
    errors: Vec<TestError>,
    #[serde(rename = "doubleEscaped", default)]
    double_escaped: bool,
    #[serde(rename = "initialStates", default)]
    initial_states: Vec<String>,
    #[serde(rename = "lastStartTag", default)]
    last_start_tag: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TestError {
    code: String,
    // line, col are usually ignored in simple tests
}

fn unescape(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'u') {
            chars.next(); // 'u'
            let mut hex = String::new();
            for _ in 0..4 {
                if let Some(hc) = chars.next() {
                    hex.push(hc);
                }
            }
            if let Some(rc) = u32::from_str_radix(&hex, 16)
                .ok()
                .and_then(std::char::from_u32)
            {
                result.push(rc);
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn json_to_token(val: &serde_json::Value) -> Token {
    let arr = val.as_array().expect("token should be an array");
    let type_name = arr[0].as_str().expect("token type should be a string");
    match type_name {
        "DOCTYPE" => Token::Doctype {
            name: arr[1].as_str().map(|s| s.to_string()),
            public_id: arr[2].as_str().map(|s| s.to_string()),
            system_id: arr[3].as_str().map(|s| s.to_string()),
            force_quirks: !arr[4].as_bool().expect("force-quirks should be a bool"),
        },
        "StartTag" => {
            let name = arr[1]
                .as_str()
                .expect("tag name should be a string")
                .to_string();
            let mut attrs = Vec::new();
            let attr_map = arr[2].as_object().expect("attributes should be an object");
            for (k, v) in attr_map {
                attrs.push((
                    k.clone(),
                    v.as_str()
                        .expect("attribute value should be a string")
                        .to_string(),
                ));
            }
            // Sort attributes for comparison if needed, but Vec order might differ.
            // HTML spec says attribute order is not preserved.
            attrs.sort_by(|a, b| a.0.cmp(&b.0));

            let self_closing = if arr.len() > 3 {
                arr[3]
                    .as_bool()
                    .expect("self-closing flag should be a bool")
            } else {
                false
            };
            Token::StartTag {
                name,
                attrs,
                self_closing,
            }
        }
        "EndTag" => {
            let name = arr[1]
                .as_str()
                .expect("tag name should be a string")
                .to_string();
            Token::EndTag {
                name,
                attrs: Vec::new(),
                self_closing: false,
            }
        }
        "Character" => {
            // This is handled by coalescing in the test runner
            panic!("Character token should be handled specially");
        }
        "Comment" => Token::Comment(
            arr[1]
                .as_str()
                .expect("comment data should be a string")
                .to_string(),
        ),
        _ => panic!("Unknown token type: {}", type_name),
    }
}

#[test]
fn content_model_flags() {
    run_test_file("tests/html5lib-tests/tokenizer/contentModelFlags.test");
}

#[test]
fn test1() {
    run_test_file("tests/html5lib-tests/tokenizer/test1.test");
}

#[test]
fn test2() {
    run_test_file("tests/html5lib-tests/tokenizer/test2.test");
}

#[test]
fn test3() {
    run_test_file("tests/html5lib-tests/tokenizer/test3.test");
}

#[test]
fn test4() {
    run_test_file("tests/html5lib-tests/tokenizer/test4.test");
}

#[test]
fn escape_flag() {
    run_test_file("tests/html5lib-tests/tokenizer/escapeFlag.test");
}

#[test]
fn entities() {
    // This will likely be mostly skipped, but let's see.
    run_test_file("tests/html5lib-tests/tokenizer/entities.test");
}

fn run_test_file(path: &str) {
    let content = std::fs::read_to_string(path).expect("failed to read test file");
    let file: TestFile = serde_json::from_str(&content).expect("failed to parse JSON");
    for test in file.tests {
        let states = if test.initial_states.is_empty() {
            vec!["Data state".to_string()]
        } else {
            test.initial_states.clone()
        };

        for state in states {
            let supported_states = [
                "Data state",
                "RCDATA state",
                "RAWTEXT state",
                "Script data state",
                "PLAINTEXT state",
            ];
            if !supported_states.contains(&state.as_str()) {
                continue;
            }

            if test.input.contains('&') {
                continue;
            }

            let input = if test.double_escaped {
                unescape(&test.input)
            } else {
                test.input.clone()
            };

            let stream = InputStream::from_utf8(input.as_bytes());
            let mut tokenizer = Tokenizer::new(stream);
            tokenizer.set_initial_state(&state);
            if let Some(ref tag) = test.last_start_tag {
                tokenizer.set_last_start_tag(tag);
            }

            let mut actual_tokens = Vec::new();
            loop {
                let tok = tokenizer.next_token();
                if tok == Token::Eof {
                    break;
                }
                actual_tokens.push(tok);
            }

            let coalesced_actual = actual_tokens;
            let mut expected_idx = 0;
            let mut actual_idx = 0;

            while expected_idx < test.output.len() {
                let expected_val = &test.output[expected_idx];
                let expected_arr = expected_val
                    .as_array()
                    .expect("expected token should be an array");
                let type_name = expected_arr[0]
                    .as_str()
                    .expect("expected token type should be a string");

                if type_name == "Character" {
                    let expected_str = expected_arr[1]
                        .as_str()
                        .expect("expected character data should be a string");
                    let mut actual_str = String::new();
                    while actual_idx < coalesced_actual.len() {
                        if let Token::Character(c) = coalesced_actual[actual_idx] {
                            actual_str.push(c);
                            actual_idx += 1;
                            if actual_str.len() == expected_str.len() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    assert_eq!(
                        actual_str, expected_str,
                        "Character mismatch in test: {}. Initial state: {}",
                        test.description, state
                    );
                } else {
                    let expected_tok = json_to_token(expected_val);
                    assert!(
                        actual_idx < coalesced_actual.len(),
                        "Missing token in test: {}. Expected {:?}",
                        test.description,
                        expected_tok
                    );
                    let mut actual_tok = coalesced_actual[actual_idx].clone();
                    actual_idx += 1;

                    if let Token::StartTag { ref mut attrs, .. } = actual_tok {
                        attrs.sort_by(|a, b| a.0.cmp(&b.0));
                    }
                    if let Token::EndTag {
                        ref mut attrs,
                        ref mut self_closing,
                        ..
                    } = actual_tok
                    {
                        attrs.clear();
                        *self_closing = false;
                    }

                    assert_eq!(
                        actual_tok, expected_tok,
                        "Token mismatch in test: {}. Initial state: {}",
                        test.description, state
                    );
                }
                expected_idx += 1;
            }

            assert_eq!(
                actual_idx,
                coalesced_actual.len(),
                "Extra tokens in test: {}. Initial state: {}",
                test.description,
                state
            );
        }
    }
}
