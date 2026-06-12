/// Represents an image candidate parsed from the `srcset` attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCandidate {
    pub url: String,
    /// Candidate's parsed density. If `w` descriptor, it defaults to 1.0 here
    /// and is normalized dynamically during selection using `effective_px`.
    /// If `x` descriptor, it holds that value. Otherwise, defaults to 1.0.
    pub density: f32,
    /// Raw width value if a `w` descriptor is specified (e.g., `480w` -> `Some(480)`).
    pub w_descriptor: Option<u32>,
}

/// Parses the `srcset` attribute to extract candidate images.
/// Empty/malformed candidates or invalid descriptors are skipped.
pub fn parse_srcset(srcset: &str) -> Vec<ImageCandidate> {
    let mut candidates = Vec::new();

    // TODO(spec): Finer per-descriptor parse-error reporting is left for the future.
    for part in srcset.split(',') {
        let tokens: Vec<&str> = part.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        let url = tokens[0].to_string();
        if tokens.len() == 1 {
            candidates.push(ImageCandidate {
                url,
                density: 1.0,
                w_descriptor: None,
            });
        } else if tokens.len() == 2 {
            let desc = tokens[1];
            if (desc.ends_with('w') || desc.ends_with('W'))
                && let Ok(w) = desc[..desc.len() - 1].parse::<u32>()
                && w > 0
            {
                candidates.push(ImageCandidate {
                    url,
                    density: 1.0,
                    w_descriptor: Some(w),
                });
            } else if (desc.ends_with('x') || desc.ends_with('X'))
                && let Ok(x) = desc[..desc.len() - 1].parse::<f32>()
                && x >= 0.0
            {
                candidates.push(ImageCandidate {
                    url,
                    density: x,
                    w_descriptor: None,
                });
            }
        }
    }

    let has_width = candidates.iter().any(|c| c.w_descriptor.is_some());
    let has_density = candidates.iter().any(|c| c.w_descriptor.is_none());
    if has_width && has_density {
        Vec::new()
    } else {
        candidates
    }
}

/// Selects the best image candidate based on device pixel ratio and effective display width (effective_px).
/// Finds the candidate with the smallest effective density that is >= device_pixel_ratio.
/// If no such candidate exists, falls back to the candidate with the maximum effective density.
/// If candidates slice is empty, returns None.
pub fn select_candidate(
    candidates: &[ImageCandidate],
    device_pixel_ratio: f32,
    effective_px: u32,
) -> Option<&ImageCandidate> {
    if candidates.is_empty() {
        return None;
    }

    let eff_px = if effective_px == 0 { 1 } else { effective_px };

    // Resolve effective densities for all candidates
    let resolved: Vec<(usize, f32)> = candidates
        .iter()
        .enumerate()
        .map(|(idx, cand)| {
            let density = match cand.w_descriptor {
                Some(w) => w as f32 / eff_px as f32,
                None => cand.density,
            };
            (idx, density)
        })
        .collect();

    // Find candidates with density >= device_pixel_ratio
    let eligible: Vec<&(usize, f32)> = resolved
        .iter()
        .filter(|(_, d)| *d >= device_pixel_ratio)
        .collect();

    let chosen_idx = if !eligible.is_empty() {
        // Find the one with the minimum density
        let mut min_item = eligible[0];
        for item in eligible.iter().skip(1) {
            if item.1 < min_item.1 {
                min_item = item;
            }
        }
        min_item.0
    } else {
        // Fallback: find the one with the maximum density
        let mut max_item = &resolved[0];
        for item in resolved.iter().skip(1) {
            if item.1 > max_item.1 {
                max_item = item;
            }
        }
        max_item.0
    };

    Some(&candidates[chosen_idx])
}

fn parse_px(tok: &str) -> Option<u32> {
    let trimmed = tok.trim().to_lowercase();
    if trimmed.ends_with("px") {
        let numeric_part = trimmed.get(..trimmed.len() - 2)?;
        if let Ok(val) = numeric_part.parse::<f32>()
            && val.is_finite()
            && val >= 0.0
        {
            return Some(val.round() as u32);
        }
    }
    None
}

fn condition_matches(cond: &str, vw: u32) -> bool {
    let Some(inner) = cond.strip_prefix('(') else {
        return false;
    };
    let Some(inner) = inner.strip_suffix(')') else {
        return false;
    };
    let inner = inner.trim();

    if let Some((feature, value_str)) = inner.split_once(':') {
        let feature = feature.trim().to_lowercase();
        let value_str = value_str.trim();
        if let Some(n) = parse_px(value_str) {
            if feature == "max-width" {
                return vw <= n;
            } else if feature == "min-width" {
                return vw >= n;
            }
        }
    }
    false
}

/// Resolves the effective display width in pixels from the `sizes` attribute.
/// If `sizes` is None, or invalid, falls back to `viewport_width`.
pub fn resolve_sizes(sizes: Option<&str>, viewport_width: u32) -> u32 {
    let sizes_str = match sizes {
        Some(s) => s,
        None => return viewport_width,
    };

    for entry in sizes_str.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        let (cond, rest) = if entry.starts_with('(') {
            if let Some(rparen_idx) = entry.find(')') {
                let cond_str = entry.get(0..=rparen_idx);
                let rest_str = entry.get(rparen_idx + 1..);
                match (cond_str, rest_str) {
                    (Some(c), Some(r)) => (Some(c), r.trim()),
                    _ => (Some(""), ""), // Unparseable, treat condition as non-matching.
                }
            } else {
                (Some(""), "") // Unparseable, treat condition as non-matching.
            }
        } else {
            (None, entry)
        };

        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let Some(last_token) = tokens.last() else {
            continue;
        };

        let Some(size_val) = parse_px(last_token) else {
            continue;
        };

        let matches = match cond {
            Some(c) => condition_matches(c, viewport_width),
            None => true,
        };

        if matches {
            return size_val;
        }
    }

    viewport_width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srcset_basic() {
        let candidates = parse_srcset("a.png 1x, b.png 2x");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].url, "a.png");
        assert_eq!(candidates[0].density, 1.0);
        assert_eq!(candidates[0].w_descriptor, None);
        assert_eq!(candidates[1].url, "b.png");
        assert_eq!(candidates[1].density, 2.0);
        assert_eq!(candidates[1].w_descriptor, None);
    }

    #[test]
    fn test_parse_srcset_w_descriptors() {
        let candidates = parse_srcset("a.png 480w, b.png 960w");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].url, "a.png");
        assert_eq!(candidates[0].w_descriptor, Some(480));
        assert_eq!(candidates[1].url, "b.png");
        assert_eq!(candidates[1].w_descriptor, Some(960));
    }

    #[test]
    fn test_parse_srcset_no_descriptor() {
        let candidates = parse_srcset("a.png");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].url, "a.png");
        assert_eq!(candidates[0].density, 1.0);
        assert_eq!(candidates[0].w_descriptor, None);
    }

    #[test]
    fn test_parse_srcset_empty_garbage() {
        assert!(parse_srcset("").is_empty());
        assert!(parse_srcset(",").is_empty());
        assert!(parse_srcset("   ").is_empty());
        assert!(parse_srcset(", ,").is_empty());

        // Single garbage descriptor or extra tokens - should be skipped/panic-free
        let candidates = parse_srcset("a.png invalid, b.png 2x 3x, c.png 2x");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].url, "c.png");
        assert_eq!(candidates[0].density, 2.0);
    }

    #[test]
    fn test_select_candidate_basic() {
        let candidates = vec![
            ImageCandidate {
                url: "a.png".to_string(),
                density: 1.0,
                w_descriptor: None,
            },
            ImageCandidate {
                url: "b.png".to_string(),
                density: 2.0,
                w_descriptor: None,
            },
        ];

        // DPR = 1.0 -> 1x
        let selected = select_candidate(&candidates, 1.0, 1000);
        assert_eq!(selected.unwrap().url, "a.png");

        // DPR = 2.0 -> 2x
        let selected = select_candidate(&candidates, 2.0, 1000);
        assert_eq!(selected.unwrap().url, "b.png");

        // DPR = 1.5 -> 2x (smallest density >= DPR)
        let selected = select_candidate(&candidates, 1.5, 1000);
        assert_eq!(selected.unwrap().url, "b.png");

        // DPR = 3.0 -> fallback to max density (2x)
        let selected = select_candidate(&candidates, 3.0, 1000);
        assert_eq!(selected.unwrap().url, "b.png");
    }

    #[test]
    fn test_resolve_sizes_basic() {
        assert_eq!(resolve_sizes(Some("600px"), 1280), 600);
        assert_eq!(resolve_sizes(None, 1280), 1280);
        assert_eq!(resolve_sizes(Some("garbage"), 1280), 1280);

        // Media conditional sizes fallback parsing
        assert_eq!(
            resolve_sizes(Some(" (max-width: 600px) 200px, 100px"), 1280),
            100
        );
    }

    #[test]
    fn test_resolve_sizes_media_conditioned() {
        // (max-width: 600px) 200px, 100px at vw=500 -> 200 (condition matches).
        assert_eq!(
            resolve_sizes(Some("(max-width: 600px) 200px, 100px"), 500),
            200
        );

        // (max-width: 600px) 200px, 100px at vw=1280 -> 100 (falls through to default).
        assert_eq!(
            resolve_sizes(Some("(max-width: 600px) 200px, 100px"), 1280),
            100
        );

        // (min-width: 900px) 400px, 50px at vw=1000 -> 400.
        assert_eq!(
            resolve_sizes(Some("(min-width: 900px) 400px, 50px"), 1000),
            400
        );

        // (min-width: 900px) 400px, 50px at vw=300 -> 50 (default).
        assert_eq!(
            resolve_sizes(Some("(min-width: 900px) 400px, 50px"), 300),
            50
        );

        // (max-width: 600px) 200px at vw=1280 (no default entry, no match) -> 1280 (viewport fallback).
        assert_eq!(resolve_sizes(Some("(max-width: 600px) 200px"), 1280), 1280);

        // multiple conditions: (max-width:400px) 100px, (max-width:800px) 300px, 600px at vw=700 -> 300 (first matching is the second entry).
        assert_eq!(
            resolve_sizes(
                Some("(max-width:400px) 100px, (max-width:800px) 300px, 600px"),
                700
            ),
            300
        );
    }

    #[test]
    fn test_select_candidate_w_descriptor() {
        let candidates = vec![
            ImageCandidate {
                url: "a.png".to_string(),
                density: 1.0,
                w_descriptor: Some(480),
            },
            ImageCandidate {
                url: "b.png".to_string(),
                density: 1.0,
                w_descriptor: Some(960),
            },
        ];

        // effective_px = 480 -> density of a.png = 1.0, b.png = 2.0
        // DPR = 1.0 -> 480w
        let selected = select_candidate(&candidates, 1.0, 480);
        assert_eq!(selected.unwrap().url, "a.png");

        // DPR = 2.0 -> 960w
        let selected = select_candidate(&candidates, 2.0, 480);
        assert_eq!(selected.unwrap().url, "b.png");
    }

    #[test]
    fn test_parse_srcset_mixed_w_and_x_invalid() {
        let candidates = parse_srcset("a.png 480w, b.png 2x");
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_parse_srcset_mixed_w_and_default_invalid() {
        let candidates = parse_srcset("a.png 480w, b.png");
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_parse_srcset_all_w_still_valid() {
        let candidates = parse_srcset("a.png 480w, b.png 960w");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].url, "a.png");
        assert_eq!(candidates[0].w_descriptor, Some(480));
        assert_eq!(candidates[1].url, "b.png");
        assert_eq!(candidates[1].w_descriptor, Some(960));
    }

    #[test]
    fn test_parse_srcset_all_x_still_valid() {
        let candidates = parse_srcset("a.png 1x, b.png 2x");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].url, "a.png");
        assert_eq!(candidates[0].density, 1.0);
        assert_eq!(candidates[1].url, "b.png");
        assert_eq!(candidates[1].density, 2.0);
    }
}
