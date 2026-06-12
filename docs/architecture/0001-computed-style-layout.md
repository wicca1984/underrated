# ADR 0001: Categorized Arc-shared ComputedStyle Layout

- **Status**: Proposed
- **Date**: June 12, 2026
- **Authors**: Gemini CLI Worker (t0443)
- **Deciders**: PdM / Browser Engine Architects
- **Milestone**: MS-CSS-Architecture

## 1. Context & Problem Statement

Today, the `underrated` CSS engine represents computed styles on a per-node basis using a dictionary map of string property names to parsed CSS values. Specifically, `src/style/mod.rs` defines `ComputedStyle` as follows:

```rust
#[derive(Debug, Default, Clone)]
pub struct ComputedStyle {
    properties: HashMap<String, CssValue>,
    opacity_compat: std::cell::OnceCell<CssValue>,
}
```

While this representation is simple, flexible, and has served the engine well during initial development, it exhibits several critical bottlenecks that prevent it from scaling to large, production-grade DOM trees:

1. **High Memory Overhead per Node**:
   - Every `ComputedStyle` instance contains a heap-allocated `HashMap`. A standard `HashMap` carries a default bucket capacity, metadata flags, and hash state overhead.
   - Property names are stored as individual heap-allocated `String` keys (e.g., `"color"`, `"margin-left"`), duplicating these static names millions of times in memory for a typical large document.
   - For every element in the tree, we store its own separate collection of properties, even if they are at their default (initial) values.

2. **Costly Inheritance Resolution**:
   - In `compute_node_style` in `src/style/mod.rs`, style inheritance is resolved by iterating over parent properties and deep cloning them:
     ```rust
     for (prop, val) in &parent_style.properties {
         if is_inherited_property(prop) && !properties.contains_key(prop) {
             properties.insert(prop.clone(), val.clone());
         }
     }
     ```
   - This leads to massive allocation and copying. For a deep DOM tree, properties are copied and re-allocated over and over down the hierarchy, leading to $O(N \cdot P)$ memory growth where $N$ is the number of nodes and $P$ is the number of inherited properties.

3. **Poor Cache Locality and Performance**:
   - Traversal of elements during layout or painting requires querying styles. Looking up properties like `style.get("display")` involves a string hashing operation, hash table bucket traversal, and chasing heap pointers.
   - Because each property lookup goes through a dynamic hash map and heap-allocated nodes, cache-locality is degraded, causing frequent CPU L1/L2 cache misses during layout passes.
   - The string lookups happen in hot loops (e.g., inline formatting, text rendering, paint list generation), introducing severe CPU bottlenecks.

Therefore, we need a high-performance, robust, and permanent foundation for `ComputedStyle` that scales to large DOMs without memory blowup or cache misses, modeled after industrial-strength styling engines like Servo (Stylo) and WebKit.

## 2. Decision: Categorized, Arc-Shared ComputedStyle

We will redesign `ComputedStyle` by transitioning from a dynamic `HashMap<String, CssValue>` to a **categorized, Arc-shared, strongly-typed layout**.

### 2.1 Categorization of Properties

Rather than a single flat dictionary, properties are clustered into logical structs categorized by functionality and inheritance. Following Stylo's architecture, we divide them into:
1. **Inherited categories**: Groups of properties that are inherited by default.
2. **Reset categories**: Groups of properties that are *not* inherited (they reset to their initial values on each element).

This partition yields two massive optimizations:
- **Zero-allocation inheritance**: To inherit an entire group of properties (e.g., all font and text styling), a child element simply clones a thread-safe atomic pointer (`Arc`) to the parent's group.
- **Shared initial values**: Elements that use default values for a category share a single global, statically-allocated or thread-safe reference to the category's initial state, requiring zero heap allocations.

The table below maps **every** CSS property currently defined in `src/css/property.rs` and supported by `underrated` into exactly one structured category:

| Category Struct | Type | Properties Contained |
| :--- | :--- | :--- |
| **`InheritedText`** | Inherited | `color`, `font-family`, `font-size`, `font-style`, `font-weight`, `line-height`, `text-align`, `letter-spacing`, `word-spacing`, `white-space`, `direction`, `text-transform`, `font-variant`, `font-stretch`, `text-indent`, `word-break`, `overflow-wrap`, `text-align-last`, `tab-size`, `hyphens` |
| **`InheritedList`** | Inherited | `list-style-type`, `list-style-position`, `list-style-image` |
| **`InheritedTable`** | Inherited | `caption-side`, `border-collapse`, `border-spacing` |
| **`InheritedUI`** | Inherited | `cursor`, `quotes` |
| **`InheritedEffects`**| Inherited | `visibility`, `empty-cells` |
| **`ResetBox`** | Reset | `display`, `width`, `height`, `position`, `float`, `clear`, `overflow`, `z-index`, `box-sizing`, `min-width`, `min-height`, `max-width`, `max-height`, `vertical-align`, `object-fit`, `pointer-events` |
| **`ResetSurround`** | Reset | `margin-top`, `margin-right`, `margin-bottom`, `margin-left`, `margin-block-start`, `margin-block-end`, `padding-top`, `padding-right`, `padding-bottom`, `padding-left`, `padding-block-start`, `padding-block-end`, `border-top-width`, `border-right-width`, `border-bottom-width`, `border-left-width`, `border-top-style`, `border-right-style`, `border-bottom-style`, `border-left-style`, `border-top-color`, `border-right-color`, `border-bottom-color`, `border-left-color`, `border-top-left-radius`, `border-top-right-radius`, `border-bottom-right-radius`, `border-bottom-left-radius`, `top`, `right`, `bottom`, `left` |
| **`ResetBackground`** | Reset | `background-color`, `background-image`, `background-repeat`, `background-position`, `background-size`, `background-attachment` |
| **`ResetFlex`** | Reset | `flex-grow`, `flex-shrink`, `flex-basis`, `flex-direction`, `flex-wrap`, `justify-content`, `align-items`, `align-self`, `order` |
| **`ResetTable`** | Reset | `table-layout` |
| **`ResetEffects`** | Reset | `opacity`, `outline-width`, `outline-style`, `outline-color`, `transition-duration`, `transition-property`, `text-decoration-line`, `text-decoration-color`, `text-decoration-style`, `text-overflow` |

The structure of the new `ComputedStyle` is:

```rust
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    // Inherited Style categories
    pub inherited_text: Arc<InheritedText>,
    pub inherited_list: Arc<InheritedList>,
    pub inherited_table: Arc<InheritedTable>,
    pub inherited_ui: Arc<InheritedUI>,
    pub inherited_effects: Arc<InheritedEffects>,

    // Reset (Non-inherited) Style categories
    pub reset_box: Arc<ResetBox>,
    pub reset_surround: Arc<ResetSurround>,
    pub reset_background: Arc<ResetBackground>,
    pub reset_flex: Arc<ResetFlex>,
    pub reset_table: Arc<ResetTable>,
    pub reset_effects: Arc<ResetEffects>,
}
```

### 2.2 Sharing Architecture: Arc and Copy-on-Write

1. **The Single-Thread vs. Multi-Thread Trade-off**:
   - *Single-threaded (`Rc`)*: Offers lowest overhead because pointer increment/decrement operations skip atomic memory barriers.
   - *Multi-threaded (`Arc`)*: Introduces slight atomic reference counting overhead, but acts as a "no-going-back" foundation, making the style data structures thread-safe (`Send + Sync`).
   - *Recommendation*: **We adopt `Arc`**. Browser styling is exceptionally parallelizable (e.g., parallel DOM style resolution using Rayon). Utilizing `Arc` ensures that style resolution and subsequent multi-threaded layout and paint pipelines are fully unblocked without a future major refactoring.

2. **Copy-on-Write (COW) Semantics**:
   - Mutating a specific category utilizes Rust's `Arc::make_mut` or `Arc::get_mut`.
   - If the element uniquely owns the `Arc` (refcount is 1), `make_mut` provides direct, in-place mutable access without any allocation.
   - If the element shares the `Arc` with its parent or siblings (refcount > 1), `make_mut` transparently clones only that single category struct and redirects the element's pointer, leaving all other shared sibling categories untouched.
   ```rust
   impl ComputedStyle {
       pub fn set_color(&mut self, new_color: Color) {
           // If shared, only InheritedText is cloned. All other Arcs remain shared!
           Arc::make_mut(&mut self.inherited_text).color = new_color;
       }
   }
   ```

3. **Style Sharing / Pointer Equality**:
   - If two elements in the DOM share identical styles (e.g., list items with no inline styles or override rules), their `ComputedStyle` instances are identical collections of `Arc` pointers.
   - Checking style equality between two elements (crucial for optimizing layout dirty tracking) becomes a fast series of pointer comparison operations ($O(1)$ pointer equality check) instead of deep recursive traversals or hash comparisons.

## 3. Static Dispatch & Declarative Macros

To eliminate runtime string comparisons (e.g. `is_inherited_property("color")` and key lookups), we introduce a compile-time static dispatch system driven by declarative macros.

### 3.1 Compile-time PropertyId Enum

We define a strongly-typed `PropertyId` enum representing all longhands:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum PropertyId {
    Color,
    FontSize,
    MarginTop,
    // ...
}
```

### 3.2 Declarative Macro: `define_properties!`

We replace the manual static tables in `src/css/property.rs` and hardcoded lists with a declarative macro. This macro represents the single source of truth for the CSS engine. It automatically generates the `PropertyId` enum, category structs, parsing dispatch, initial values, and inheritance checks.

```rust
macro_rules! define_properties {
    (
        $(
            $name:ident {
                inherited: $inherited:expr,
                type: $value_type:ty,
                initial: $initial_expr:expr,
                category: $category:ident,
                field: $field:ident,
                parse_fn: $parse_fn:path,
            }
        )*
    ) => {
        // 1. Generate PropertyId Enum
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum PropertyId {
            $( [<$name:camel>] ),*
        }

        // 2. Generate Category Structs
        // (Grouping fields by category)
        // ...

        // 3. Generate Lookup Table
        pub fn lookup_property(name: &str) -> Option<PropertyId> {
            // High-performance static match mapping string to PropertyId
            match name {
                $( stringify!($name) => Some(PropertyId::[<$name:camel>]), )*
                _ => None,
            }
        }
    };
}
```

This macro completely replaces the runtime `HashMap` lookup and string checks with compiler-optimized `match` statements, reducing property lookup down to a machine register comparison or dense array index access.

## 4. Bitflags for Inheritance & Dirty Tracking

To support highly optimized restyle propagation and layout invalidation, we introduce a bit-parallel metadata layer using the `bitflags` crate.

1. **Explicitly Set Flags (`explicitly_set_properties`)**:
   - Each `ComputedStyle` holds a bitmask where each bit corresponds to a `PropertyId`.
   - When a property is explicitly set by a CSS rule, inline style, or HTML hint, its bit is flipped to `1`.
   - During the cascade, inheritance resolution can check the bitmask in a single CPU cycle to determine if it should copy the parent's value or use the inherited `Arc`.

2. **Category Dirty Flags (`restyle_hint`)**:
   - We define a bitmask representing the 11 categories of style:
     ```rust
     bitflags::bitflags! {
         pub struct RestyleHint: u16 {
             const TEXT       = 1 << 0;
             const LIST       = 1 << 1;
             const SURROUND   = 1 << 2;
             const BACKGROUND = 1 << 3;
             const FLEX       = 1 << 4;
             // ...
         }
     }
     ```
   - When a dynamic DOM change occurs (e.g., via script changing a class), the restyle engine calculates the diff between the old style and new style.
   - Instead of checking every field of a giant struct, it does a bitwise XOR of the category `Arc` pointers!
   - If `old_style.reset_surround` and `new_style.reset_surround` have different pointers, the `SURROUND` dirty bit is set.
   - The layout engine uses this mask: if only `BACKGROUND` is dirty, layout is skipped entirely, and only painting is invalidated, saving massive CPU cycles on dynamic updates.

## 5. Incremental Migration Plan

A complete "big bang" rewrite of the style engine would halt feature development and introduce high risk. We will execute the migration incrementally, maintaining a compiling, green-test codebase at every step.

```
+-------------------------------------------------------------+
| Phase A: Implement internal categories under stable getters |
| - Implement Structs (InheritedText, ResetBox, etc.)         |
| - Bridge legacy ComputedStyle::get(&self, name)             |
| - Verify: Entire test suite remains green                   |
+-------------------------------------------------------------+
                              |
                              v
+-------------------------------------------------------------+
| Phase B: Standalone Prototype & Microbenchmark              |
| - Build macro-driven prototype in src/css/prototype/        |
| - Benchmark: Legacy vs. Arc-Shared with 10k nodes           |
| - Verify: Measure & validate memory and cache performance   |
+-------------------------------------------------------------+
                              |
                              v
+-------------------------------------------------------------+
| Phase C: Gradual Property Migration                         |
| - Migrate property fields category-by-category             |
| - Update getters in ComputedStyle to resolve from structs   |
| - Verify: Incremental cargo test run is green               |
+-------------------------------------------------------------+
                              |
                              v
+-------------------------------------------------------------+
| Phase D: Deprecate Legacy HashMap & String Lookups          |
| - Eliminate HashMap in ComputedStyle                        |
| - Replace get(&str) with get(PropertyId) in layout / paint  |
| - Remove old code, finalize compiler optimizations          |
+-------------------------------------------------------------+
```

### Phase A: Internal Struct Foundation
- Keep `ComputedStyle`'s public API completely intact:
  ```rust
  pub fn get(&self, property: &str) -> Option<&CssValue>;
  pub fn insert(&mut self, property: String, value: CssValue);
  ```
- Introduce the new category structs as fields in `ComputedStyle`.
- Implement a bridge within `ComputedStyle::get` and `insert` that delegates queries of migrated properties to the new structs, while falling back to the legacy `HashMap` for unmigrated ones.
- **Verification**: Run `cargo test` to ensure zero regressions across the codebase.

### Phase B: Prototype & Benchmark (Next Task)
- Implement the macro-driven static dispatch, `PropertyId` enum, and the full `Arc` copy-on-write sharing model in a standalone directory: `src/css/prototype/`.
- Run microbenchmarks on synthetic, deeply-nested DOM trees of varying sizes ($N = 1,000$ to $N = 50,000$) simulating style cascades.
- **Verification**: Validate that memory usage drops by $>80\%$ and styling resolution speed increases by $>5\text{x}$ compared to the legacy `HashMap`.

### Phase C: Category-by-Category Migration
- Systematically move groups of properties into their categories:
  1. Font & Text properties (`color`, `font-size`, etc.) -> `InheritedText`.
  2. Spacing and Box model (`margin`, `padding`, `border`, etc.) -> `ResetSurround`.
  3. Rest of categories sequentially.
- With each category, delete the respective code handling them from the legacy fallbacks.

### Phase D: Cleanup and Switch to Static Dispatch
- Completely remove `HashMap<String, CssValue>` and `opacity_compat`.
- Refactor all external call sites in `layout/`, `paint/`, and `dom/` from string-based lookups (`style.get("display")`) to direct category access (`style.reset_box.display`) or enum lookups (`style.get(PropertyId::Display)`).

## 6. Alternatives Considered & Rejected

### Alternative A: Naive Heap-Allocated Map per Node (Status Quo)
- *Rejected*: The current implementation suffers from extreme memory bloating due to individual string keys and map overhead. Deep-cloning styles for inheritance results in $O(N \cdot P)$ memory overhead and forces intensive heap allocation activity, degrading garbage collection and allocation efficiency.

### Alternative B: One Giant Flat Struct per Node (No Sharing)
- *Rejected*: Storing all ~100+ CSS properties in a single flat struct per element would avoid hash-map lookups and string allocation. However, because most elements only customize 3-5 properties and inherit the rest, copying a giant structure (~1.5KB) for every single node is incredibly wasteful. It fails to leverage the fact that elements naturally share large clusters of style values with their parent and siblings.

### Alternative C: Property Key Interning (String ID mapping)
- *Rejected*: While mapping property names to an integer `StringId` or `Atom` eliminates duplicate string allocations, it does not solve the fundamental problem. Every element would still maintain its own individual map/table of properties, resulting in high storage overhead. It does not provide $O(1)$ reference sharing for inheritance, which is the single biggest performance lever for CSS styling engines.

## 7. Consequences & Trade-offs

### Wins
1. **Unprecedented Memory Efficiency**: Style inheritance is reduced from deep-cloning values to an $O(1)$ atomic pointer copy. Shared styling across thousands of nodes requires zero extra memory.
2. **Deterministic, $O(1)$ Lookups**: Replaces dynamic hash lookups with compile-time checked direct field access or static array indexing.
3. **Optimized Change Propagation**: Bitwise XOR of category pointer addresses determines if style categories changed, drastically optimizing dynamic restyling and invalidation passes.
4. **Concurrency-Ready**: Thread-safe `Arc` allocations unblock future parallelization of the style and layout pipelines.

### Trade-offs & Mitigations
1. **Atomic Reference Count Overhead**: Modifying or cloning `Arc` pointers involves atomic operations which carry slight overhead compared to non-atomic `Rc`.
   *Mitigation*: Styling is performed once per frame / resolution sweep and is highly read-heavy. The benefits of $O(1)$ sharing and parallel-read safety far outweigh the negligible cost of atomic operations.
2. **Copy-on-Write Overhead**: Mutating a property inside a shared category requires cloning the category struct.
   *Mitigation*: These structs are extremely small and cache-friendly (e.g., `InheritedList` contains only 3 properties). Cloning a small, fixed-size contiguous struct is extremely fast and occurs only during the initial cascade or dynamic style changes, never during read-intensive layout or painting passes.
3. **Declarative Complexity**: Generating code via macros adds build-time complexity and makes backtraces slightly harder to read inside macro-expansion code.
   *Mitigation*: The macro will be clearly structured and fully documented, isolating the codegen to the `src/css/property.rs` layer, keeping the rest of the style engine idiomatic and clean.
