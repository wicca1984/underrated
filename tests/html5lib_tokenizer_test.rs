#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::Value;
use underrated::encoding::InputStream;
use underrated::html::{Token, Tokenizer};

#[derive(Debug, PartialEq, Eq, Clone)]
enum UnifiedToken {
    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Character(String),
}

fn json_to_unified_token(val: &Value) -> Result<UnifiedToken, String> {
    let arr = val
        .as_array()
        .ok_or_else(|| "token should be an array".to_string())?;
    let type_name = arr[0]
        .as_str()
        .ok_or_else(|| "token type should be a string".to_string())?;
    match type_name {
        "DOCTYPE" => {
            let name = arr.get(1).and_then(|v| v.as_str().map(|s| s.to_string()));
            let public_id = arr.get(2).and_then(|v| v.as_str().map(|s| s.to_string()));
            let system_id = arr.get(3).and_then(|v| v.as_str().map(|s| s.to_string()));
            let correctness = arr.get(4).and_then(|v| v.as_bool()).unwrap_or(true);
            Ok(UnifiedToken::Doctype {
                name,
                public_id,
                system_id,
                force_quirks: !correctness,
            })
        }
        "StartTag" => {
            let name = arr
                .get(1)
                .and_then(|v| v.as_str())
                .ok_or("tag name missing")?
                .to_string();
            let mut attrs = Vec::new();
            if let Some(attr_map) = arr.get(2).and_then(|v| v.as_object()) {
                for (k, v) in attr_map {
                    attrs.push((
                        k.clone(),
                        v.as_str().ok_or("attr value not string")?.to_string(),
                    ));
                }
            }
            attrs.sort_by(|a, b| a.0.cmp(&b.0));
            let self_closing = arr.get(3).and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(UnifiedToken::StartTag {
                name,
                attrs,
                self_closing,
            })
        }
        "EndTag" => {
            let name = arr
                .get(1)
                .and_then(|v| v.as_str())
                .ok_or("tag name missing")?
                .to_string();
            Ok(UnifiedToken::EndTag { name })
        }
        "Comment" => {
            let data = arr
                .get(1)
                .and_then(|v| v.as_str())
                .ok_or("comment data missing")?
                .to_string();
            Ok(UnifiedToken::Comment(data))
        }
        "Character" => {
            let data = arr
                .get(1)
                .and_then(|v| v.as_str())
                .ok_or("character data missing")?
                .to_string();
            Ok(UnifiedToken::Character(data))
        }
        _ => Err(format!("Unknown token type: {}", type_name)),
    }
}

fn coalesce_unified_tokens(tokens: Vec<UnifiedToken>) -> Vec<UnifiedToken> {
    let mut coalesced = Vec::new();
    for token in tokens {
        match token {
            UnifiedToken::Character(s) => {
                if let Some(UnifiedToken::Character(last_str)) = coalesced.last_mut() {
                    last_str.push_str(&s);
                } else {
                    coalesced.push(UnifiedToken::Character(s));
                }
            }
            other => coalesced.push(other),
        }
    }
    coalesced
}

#[test]
#[allow(clippy::absurd_extreme_comparisons)]
fn test_html5lib_tokenizer_conformance() {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // We focus on test1.test and test2.test
    let files = [
        "tests/html5lib-tests/tokenizer/test1.test",
        "tests/html5lib-tests/tokenizer/test2.test",
    ];

    for file_path in &files {
        let content = std::fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read test file: {}", file_path));
        let root: Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON in file: {}", file_path));

        let tests_arr = root
            .get("tests")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("No tests array in file: {}", file_path));

        for test in tests_arr {
            let description = test
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // 1. Skip if "doubleEscaped": true
            let is_double_escaped = test
                .get("doubleEscaped")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_double_escaped {
                skipped += 1;
                continue;
            }

            // 2. Skip if "initialStates" present and contains states other than "Data state"
            if let Some(states_arr) = test.get("initialStates").and_then(|v| v.as_array()) {
                let is_only_data_state =
                    states_arr.len() == 1 && states_arr[0].as_str() == Some("Data state");
                if !is_only_data_state {
                    skipped += 1;
                    continue;
                }
            }

            let input = test
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("No input in test: {}", description));

            // Run Tokenizer
            let stream = InputStream::from_utf8(input.as_bytes());
            let mut tokenizer = Tokenizer::new(stream);
            tokenizer.set_initial_state("Data state");

            if let Some(tag_str) = test.get("lastStartTag").and_then(|v| v.as_str()) {
                tokenizer.set_last_start_tag(tag_str);
            }

            let mut actual_tokens = Vec::new();
            loop {
                let tok = tokenizer.next_token();
                if tok == Token::Eof {
                    break;
                }
                actual_tokens.push(tok);
            }

            // Convert actual tokens to UnifiedToken
            let mut actual_unified = Vec::new();
            for tok in actual_tokens {
                match tok {
                    Token::Doctype {
                        name,
                        public_id,
                        system_id,
                        force_quirks,
                    } => {
                        actual_unified.push(UnifiedToken::Doctype {
                            name,
                            public_id,
                            system_id,
                            force_quirks,
                        });
                    }
                    Token::StartTag {
                        name,
                        mut attrs,
                        self_closing,
                    } => {
                        attrs.sort_by(|a, b| a.0.cmp(&b.0));
                        actual_unified.push(UnifiedToken::StartTag {
                            name,
                            attrs,
                            self_closing,
                        });
                    }
                    Token::EndTag { name, .. } => {
                        actual_unified.push(UnifiedToken::EndTag { name });
                    }
                    Token::Comment(data) => {
                        actual_unified.push(UnifiedToken::Comment(data));
                    }
                    Token::Character(c) => {
                        actual_unified.push(UnifiedToken::Character(c.to_string()));
                    }
                    Token::Eof => {}
                }
            }

            let actual_coalesced = coalesce_unified_tokens(actual_unified);

            // Convert expected tokens to UnifiedToken
            let mut expected_tokens = Vec::new();
            if let Some(output_arr) = test.get("output").and_then(|v| v.as_array()) {
                for val in output_arr {
                    match json_to_unified_token(val) {
                        Ok(tok) => expected_tokens.push(tok),
                        Err(e) => {
                            panic!(
                                "Failed to parse expected token in test '{}': {}",
                                description, e
                            );
                        }
                    }
                }
            }
            let expected_coalesced = coalesce_unified_tokens(expected_tokens);

            if actual_coalesced == expected_coalesced {
                passed += 1;
            } else {
                failed += 1;
            }
        }
    }

    // RATCHET: BASELINE_MAX_FAILURES is locked to the actual observed failures (0).
    // Future regressions (more failures) will turn the test red. If the tokenizer
    // improves and failure count drops, this constant should be lowered accordingly,
    // never raised.
    const BASELINE_MAX_FAILURES: usize = 0;

    // Print summary to stderr as required
    eprintln!(
        "html5lib tokenizer: PASS={} FAIL={} SKIP={} (baseline FAIL<={})",
        passed, failed, skipped, BASELINE_MAX_FAILURES
    );

    assert!(
        failed <= BASELINE_MAX_FAILURES,
        "Failed count {} exceeded baseline maximum {}",
        failed,
        BASELINE_MAX_FAILURES
    );
}
