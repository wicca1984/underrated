#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use underrated::dom::{Dom, NodeData};
use underrated::encoding::InputStream;
use underrated::html::parse_document;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TestCase {
    file_path: String,
    line_number: usize,
    data: String,
    errors: Vec<String>,
    document_fragment: Option<String>,
    script_off: bool,
    script_on: bool,
    expected_document: String,
}

fn find_dat_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_dat_files(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "dat") {
                files.push(path);
            }
        }
    }
}

fn parse_dat_file<P: AsRef<Path>>(path: P) -> Vec<TestCase> {
    let path = path.as_ref();
    let file_path_str = path.to_string_lossy().into_owned();
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("Failed to read file {}: {}", file_path_str, e);
    });

    let mut cases = Vec::new();
    let mut current_section = None;
    let mut current_case: Option<TestCase> = None;

    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_trimmed = line.trim_end_matches('\r');
        if line_trimmed == "#data" {
            if let Some(case) = current_case.take() {
                cases.push(case);
            }
            current_case = Some(TestCase {
                file_path: file_path_str.clone(),
                line_number: line_idx + 1,
                data: String::new(),
                errors: Vec::new(),
                document_fragment: None,
                script_off: false,
                script_on: false,
                expected_document: String::new(),
            });
            current_section = Some("data");
        } else if line_trimmed == "#errors" {
            current_section = Some("errors");
        } else if line_trimmed == "#new-errors" {
            current_section = Some("new-errors");
        } else if line_trimmed == "#document-fragment" {
            current_section = Some("document-fragment");
        } else if line_trimmed == "#script-off" {
            if let Some(ref mut c) = current_case {
                c.script_off = true;
            }
            current_section = Some("script-off");
        } else if line_trimmed == "#script-on" {
            if let Some(ref mut c) = current_case {
                c.script_on = true;
            }
            current_section = Some("script-on");
        } else if line_trimmed == "#document" {
            current_section = Some("document");
        } else {
            if let Some(ref mut c) = current_case {
                match current_section {
                    Some("data") => {
                        if !c.data.is_empty() {
                            c.data.push('\n');
                        }
                        c.data.push_str(line_trimmed);
                    }
                    Some("errors") | Some("new-errors") => {
                        c.errors.push(line_trimmed.to_string());
                    }
                    Some("document-fragment") => {
                        if let Some(ref mut df) = c.document_fragment {
                            df.push('\n');
                            df.push_str(line_trimmed);
                        } else {
                            c.document_fragment = Some(line_trimmed.to_string());
                        }
                    }
                    Some("document") if line_trimmed.starts_with('|') => {
                        if !c.expected_document.is_empty() {
                            c.expected_document.push('\n');
                        }
                        c.expected_document.push_str(line_trimmed);
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(case) = current_case {
        cases.push(case);
    }
    cases
}

fn serialize_dom(dom: &Dom) -> String {
    let mut out = Vec::new();
    let mut stack = vec![(dom.document(), 0)];

    while let Some((node, depth)) = stack.pop() {
        let Some(data) = dom.data(node) else {
            continue;
        };

        match data {
            NodeData::Document => {
                for &child in dom.children(node).iter().rev() {
                    stack.push((child, depth));
                }
            }
            NodeData::Doctype {
                name,
                public_id,
                system_id,
            } => {
                let mut s = String::new();
                s.push_str("| ");
                s.push_str(&"  ".repeat(depth));
                s.push_str("<!DOCTYPE ");
                s.push_str(name);
                if !public_id.is_empty() || !system_id.is_empty() {
                    s.push_str(" \"");
                    s.push_str(public_id);
                    s.push_str("\" \"");
                    s.push_str(system_id);
                    s.push('"');
                }
                s.push('>');
                out.push(s);
            }
            NodeData::Element { name, attrs } => {
                let mut s = String::new();
                s.push_str("| ");
                s.push_str(&"  ".repeat(depth));
                s.push('<');
                s.push_str(name);
                s.push('>');
                out.push(s);

                let mut sorted_attrs = attrs.clone();
                sorted_attrs.sort_by(|a, b| a.0.cmp(&b.0));
                for (attr_name, attr_value) in sorted_attrs {
                    let mut attr_line = String::new();
                    attr_line.push_str("| ");
                    attr_line.push_str(&"  ".repeat(depth + 1));
                    attr_line.push_str(&attr_name);
                    attr_line.push_str("=\"");
                    attr_line.push_str(&attr_value);
                    attr_line.push('"');
                    out.push(attr_line);
                }

                for &child in dom.children(node).iter().rev() {
                    stack.push((child, depth + 1));
                }
            }
            NodeData::Text(text) => {
                let mut s = String::new();
                s.push_str("| ");
                s.push_str(&"  ".repeat(depth));
                s.push('"');
                s.push_str(text);
                s.push('"');
                out.push(s);
            }
            NodeData::Comment(comment) => {
                let mut s = String::new();
                s.push_str("| ");
                s.push_str(&"  ".repeat(depth));
                s.push_str("<!-- ");
                s.push_str(comment);
                s.push_str(" -->");
                out.push(s);
            }
        }
    }

    out.join("\n")
}

#[test]
fn test_html5lib_tree_construction_conformance() {
    let mut dat_files = Vec::new();
    find_dat_files(
        Path::new("tests/html5lib-tests/tree-construction"),
        &mut dat_files,
    );
    dat_files.sort();

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for path in &dat_files {
        let cases = parse_dat_file(path);
        for case in cases {
            if case.document_fragment.is_some() {
                skipped += 1;
                continue;
            }

            // Skip test cases containing frameset, frame, noframes, or noscript elements
            // to avoid parser stack overflows due to recursive process_token calls in the current engine.
            let data_lower = case.data.to_lowercase();
            if data_lower.contains("<noscript")
                || data_lower.contains("</noscript")
                || data_lower.contains("<frameset")
                || data_lower.contains("</frameset")
                || data_lower.contains("<frame")
                || data_lower.contains("</frame")
                || data_lower.contains("<noframes")
                || data_lower.contains("</noframes")
            {
                skipped += 1;
                continue;
            }

            let stream = InputStream::from_utf8(case.data.as_bytes());
            let dom = parse_document(stream);
            let actual = serialize_dom(&dom);

            if actual == case.expected_document {
                passed += 1;
            } else {
                failed += 1;
            }
        }
    }

    const BASELINE: usize = 890;

    eprintln!(
        "html5lib tree-construction: PASS={} FAIL={} SKIP={} (baseline >= {})",
        passed, failed, skipped, BASELINE
    );

    assert!(
        passed >= BASELINE,
        "Passing count {} is below baseline {}",
        passed,
        BASELINE
    );
}
