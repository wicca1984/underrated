//! Isolated Prototype of the Arc-shared ComputedStyle design (ADR 0001).
//!
//! Proves the categorized, Arc-shared ComputedStyle architecture with Style-Sharing
//! and Copy-on-Write (COW) mutations.

use std::sync::{Arc, OnceLock};

/// An inherited category of computed properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedText {
    pub color: String,
    pub font_family: String,
    pub font_size: u32,
    pub line_height: u32,
}

impl Default for InheritedText {
    fn default() -> Self {
        Self {
            color: "black".to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 16,
            line_height: 20,
        }
    }
}

/// A reset (non-inherited) category of computed properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetBox {
    pub display: u8,
    pub width: i32,
    pub height: i32,
    pub position: u8,
}

impl Default for ResetBox {
    fn default() -> Self {
        Self {
            display: 0,
            width: -1,
            height: -1,
            position: 0,
        }
    }
}

// Process-wide shared initial allocations for Style-Sharing.
static INITIAL_INHERITED_TEXT: OnceLock<Arc<InheritedText>> = OnceLock::new();
static INITIAL_RESET_BOX: OnceLock<Arc<ResetBox>> = OnceLock::new();

/// The prototype style node representing a categorized computed style.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtoComputedStyle {
    pub inherited_text: Arc<InheritedText>,
    pub reset_box: Arc<ResetBox>,
}

impl ProtoComputedStyle {
    /// Returns a new `ProtoComputedStyle` sharing process-wide initial allocations.
    pub fn initial() -> Self {
        let inherited_text = INITIAL_INHERITED_TEXT
            .get_or_init(|| Arc::new(InheritedText::default()))
            .clone();
        let reset_box = INITIAL_RESET_BOX
            .get_or_init(|| Arc::new(ResetBox::default()))
            .clone();
        Self {
            inherited_text,
            reset_box,
        }
    }

    /// Inherits properties from a parent style node.
    /// Inherited categories are cloned (pointer copy, zero-alloc).
    /// Reset categories get the fresh process-wide initial allocation.
    pub fn inherit_from(parent: &Self) -> Self {
        let reset_box = INITIAL_RESET_BOX
            .get_or_init(|| Arc::new(ResetBox::default()))
            .clone();
        Self {
            inherited_text: parent.inherited_text.clone(),
            reset_box,
        }
    }

    /// Mutates the color of the text category using copy-on-write semantics.
    pub fn set_color(&mut self, color: String) {
        Arc::make_mut(&mut self.inherited_text).color = color;
    }

    /// Mutates the width of the box category using copy-on-write semantics.
    pub fn set_width(&mut self, width: i32) {
        Arc::make_mut(&mut self.reset_box).width = width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::time::Instant;

    #[test]
    fn test_computed_style_prototype_and_benchmark() {
        const N: usize = 10_000;

        // ---------------------------------------------------------------------
        // 1. PROTOTYPE BUILD
        // ---------------------------------------------------------------------
        let start_proto = Instant::now();
        let mut proto_nodes = Vec::with_capacity(N);

        // Root node gets the process-wide initial style.
        proto_nodes.push(ProtoComputedStyle::initial());

        for i in 1..N {
            // Inherit style from parent (node i - 1).
            let mut node = ProtoComputedStyle::inherit_from(&proto_nodes[i - 1]);

            // Only every 100th node diverges.
            if i % 100 == 0 {
                node.set_color(format!("color-{}", i));
            }
            proto_nodes.push(node);
        }
        let duration_proto = start_proto.elapsed();

        // ---------------------------------------------------------------------
        // 2. BASELINE BUILD
        // ---------------------------------------------------------------------
        let start_baseline = Instant::now();
        let mut baseline_nodes = Vec::with_capacity(N);

        // Root baseline map carrying all 8 properties.
        let mut root_map = HashMap::new();
        root_map.insert("color".to_string(), "black".to_string());
        root_map.insert("font-family".to_string(), "sans-serif".to_string());
        root_map.insert("font-size".to_string(), "16".to_string());
        root_map.insert("line-height".to_string(), "20".to_string());
        root_map.insert("display".to_string(), "0".to_string());
        root_map.insert("width".to_string(), "-1".to_string());
        root_map.insert("height".to_string(), "-1".to_string());
        root_map.insert("position".to_string(), "0".to_string());
        baseline_nodes.push(root_map);

        for i in 1..N {
            // In a standard style resolution, we inherit parent's properties and reset non-inherited properties.
            let parent_map = &baseline_nodes[i - 1];
            let mut map = HashMap::new();

            // Inherit text properties (copy string values)
            map.insert(
                "color".to_string(),
                parent_map.get("color").unwrap().clone(),
            );
            map.insert(
                "font-family".to_string(),
                parent_map.get("font-family").unwrap().clone(),
            );
            map.insert(
                "font-size".to_string(),
                parent_map.get("font-size").unwrap().clone(),
            );
            map.insert(
                "line-height".to_string(),
                parent_map.get("line-height").unwrap().clone(),
            );

            // Initialize reset properties to default values
            map.insert("display".to_string(), "0".to_string());
            map.insert("width".to_string(), "-1".to_string());
            map.insert("height".to_string(), "-1".to_string());
            map.insert("position".to_string(), "0".to_string());

            // Divergence
            if i % 100 == 0 {
                map.insert("color".to_string(), format!("color-{}", i));
            }
            baseline_nodes.push(map);
        }
        let duration_baseline = start_baseline.elapsed();

        // ---------------------------------------------------------------------
        // 3. MEASURE DISTINCT ALLOCATIONS
        // ---------------------------------------------------------------------
        // Measure unique InheritedText allocations in Prototype by collecting pointers.
        let mut unique_inherited_ptrs = HashSet::new();
        for node in &proto_nodes {
            let ptr = Arc::as_ptr(&node.inherited_text) as usize;
            unique_inherited_ptrs.insert(ptr);
        }
        let distinct_inherited_allocs = unique_inherited_ptrs.len();

        // Print human-readable comparison results.
        println!(
            "================================================================================"
        );
        println!("t0444 - MS-CSS-Architecture Style-Sharing Microbenchmark Results:");
        println!(
            "proto: distinct InheritedText allocs = {} (of {} nodes), build = {:.4}ms",
            distinct_inherited_allocs,
            N,
            duration_proto.as_secs_f64() * 1000.0
        );
        println!(
            "baseline: {} maps, build = {:.4}ms",
            N,
            duration_baseline.as_secs_f64() * 1000.0
        );
        println!(
            "================================================================================"
        );

        // ---------------------------------------------------------------------
        // 4. VERIFY DESIGN THESIS (ASSERTIONS)
        // ---------------------------------------------------------------------
        // Thesis: Sharing collapses 10k nodes' inherited styles to a handful of allocations.
        // N / 50 + 2 = 10000 / 50 + 2 = 202. Our expectation is exactly 100.
        assert!(
            distinct_inherited_allocs <= N / 50 + 2,
            "Style-sharing failed! Distinct allocations count ({}) is too high.",
            distinct_inherited_allocs
        );

        // Verify ResetBox sharing: since none was mutated, ALL 10k nodes share exactly ONE ResetBox allocation.
        let mut unique_reset_ptrs = HashSet::new();
        for node in &proto_nodes {
            let ptr = Arc::as_ptr(&node.reset_box) as usize;
            unique_reset_ptrs.insert(ptr);
        }
        assert_eq!(
            unique_reset_ptrs.len(),
            1,
            "Reset boxes were not fully shared process-wide!"
        );

        // Sanity check COW behavior: check that only mutated nodes diverged, and non-mutated nodes are shared.
        assert_eq!(proto_nodes[0].inherited_text.color, "black");
        assert_eq!(proto_nodes[99].inherited_text.color, "black");
        assert_eq!(proto_nodes[100].inherited_text.color, "color-100");
        assert_eq!(proto_nodes[101].inherited_text.color, "color-100");
        assert_eq!(proto_nodes[199].inherited_text.color, "color-100");
        assert_eq!(proto_nodes[200].inherited_text.color, "color-200");

        assert!(Arc::ptr_eq(
            &proto_nodes[0].inherited_text,
            &proto_nodes[99].inherited_text
        ));
        assert!(!Arc::ptr_eq(
            &proto_nodes[99].inherited_text,
            &proto_nodes[100].inherited_text
        ));
        assert!(Arc::ptr_eq(
            &proto_nodes[100].inherited_text,
            &proto_nodes[101].inherited_text
        ));
    }
}
