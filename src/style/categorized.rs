//! Production target style type per ADR 0001, introduced additively.
//! The legacy `ComputedStyle` is migrated off in later tasks.

use std::sync::{Arc, OnceLock};

/// Group of inherited text and font properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedText {
    pub color: String,
    pub font_family: String,
    pub font_size: u32,
    pub font_style: String,
    pub font_weight: String,
    pub line_height: u32,
    pub text_align: String,
    pub letter_spacing: i32,
    pub word_spacing: i32,
    pub white_space: String,
    pub direction: String,
    pub text_transform: String,
    pub font_variant: String,
    pub font_stretch: String,
    pub text_indent: i32,
    pub word_break: String,
    pub overflow_wrap: String,
    pub text_align_last: String,
    pub tab_size: u32,
    pub hyphens: String,
}

impl Default for InheritedText {
    fn default() -> Self {
        Self {
            color: "black".to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 16,
            font_style: "normal".to_string(),
            font_weight: "normal".to_string(),
            line_height: 20,
            text_align: "start".to_string(),
            letter_spacing: -1,
            word_spacing: -1,
            white_space: "normal".to_string(),
            direction: "ltr".to_string(),
            text_transform: "none".to_string(),
            font_variant: "normal".to_string(),
            font_stretch: "normal".to_string(),
            text_indent: -1,
            word_break: "normal".to_string(),
            overflow_wrap: "normal".to_string(),
            text_align_last: "auto".to_string(),
            tab_size: 8,
            hyphens: "manual".to_string(),
        }
    }
}

/// Group of inherited list properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedList {
    pub list_style_type: String,
    pub list_style_position: String,
    pub list_style_image: String,
}

impl Default for InheritedList {
    fn default() -> Self {
        Self {
            list_style_type: "disc".to_string(),
            list_style_position: "outside".to_string(),
            list_style_image: "none".to_string(),
        }
    }
}

/// Group of inherited table properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedTable {
    pub caption_side: String,
    pub border_collapse: String,
    pub border_spacing: u32,
}

impl Default for InheritedTable {
    fn default() -> Self {
        Self {
            caption_side: "top".to_string(),
            border_collapse: "separate".to_string(),
            border_spacing: 0,
        }
    }
}

/// Group of inherited UI properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedUI {
    pub cursor: String,
    pub quotes: String,
}

impl Default for InheritedUI {
    fn default() -> Self {
        Self {
            cursor: "auto".to_string(),
            quotes: "auto".to_string(),
        }
    }
}

/// Group of inherited effect properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedEffects {
    pub visibility: String,
    pub empty_cells: String,
}

impl Default for InheritedEffects {
    fn default() -> Self {
        Self {
            visibility: "visible".to_string(),
            empty_cells: "show".to_string(),
        }
    }
}

/// Group of reset box layout properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetBox {
    pub display: String,
    pub width: i32,
    pub height: i32,
    pub position: String,
    pub float: String,
    pub clear: String,
    pub overflow: String,
    pub z_index: i32,
    pub box_sizing: String,
    pub min_width: i32,
    pub min_height: i32,
    pub max_width: i32,
    pub max_height: i32,
    pub vertical_align: i32,
    pub object_fit: String,
    pub pointer_events: String,
}

impl Default for ResetBox {
    fn default() -> Self {
        Self {
            display: "inline".to_string(),
            width: -1,
            height: -1,
            position: "static".to_string(),
            float: "none".to_string(),
            clear: "none".to_string(),
            overflow: "visible".to_string(),
            z_index: 0,
            box_sizing: "content-box".to_string(),
            min_width: -1,
            min_height: -1,
            max_width: -1,
            max_height: -1,
            vertical_align: -1,
            object_fit: "fill".to_string(),
            pointer_events: "auto".to_string(),
        }
    }
}

/// Group of reset spacing and border properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetSurround {
    pub margin_top: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub margin_block_start: i32,
    pub margin_block_end: i32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub padding_block_start: i32,
    pub padding_block_end: i32,
    pub border_top_width: i32,
    pub border_right_width: i32,
    pub border_bottom_width: i32,
    pub border_left_width: i32,
    pub border_top_style: String,
    pub border_right_style: String,
    pub border_bottom_style: String,
    pub border_left_style: String,
    pub border_top_color: String,
    pub border_right_color: String,
    pub border_bottom_color: String,
    pub border_left_color: String,
    pub border_top_left_radius: i32,
    pub border_top_right_radius: i32,
    pub border_bottom_right_radius: i32,
    pub border_bottom_left_radius: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Default for ResetSurround {
    fn default() -> Self {
        Self {
            margin_top: -1,
            margin_right: -1,
            margin_bottom: -1,
            margin_left: -1,
            margin_block_start: -1,
            margin_block_end: -1,
            padding_top: -1,
            padding_right: -1,
            padding_bottom: -1,
            padding_left: -1,
            padding_block_start: -1,
            padding_block_end: -1,
            border_top_width: -1,
            border_right_width: -1,
            border_bottom_width: -1,
            border_left_width: -1,
            border_top_style: "none".to_string(),
            border_right_style: "none".to_string(),
            border_bottom_style: "none".to_string(),
            border_left_style: "none".to_string(),
            border_top_color: "currentcolor".to_string(),
            border_right_color: "currentcolor".to_string(),
            border_bottom_color: "currentcolor".to_string(),
            border_left_color: "currentcolor".to_string(),
            border_top_left_radius: -1,
            border_top_right_radius: -1,
            border_bottom_right_radius: -1,
            border_bottom_left_radius: -1,
            top: -1,
            right: -1,
            bottom: -1,
            left: -1,
        }
    }
}

/// Group of reset background properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetBackground {
    pub background_color: String,
    pub background_image: String,
    pub background_repeat: String,
    pub background_position: String,
    pub background_size: String,
    pub background_attachment: String,
}

impl Default for ResetBackground {
    fn default() -> Self {
        Self {
            background_color: "transparent".to_string(),
            background_image: "none".to_string(),
            background_repeat: "repeat".to_string(),
            background_position: "0% 0%".to_string(),
            background_size: "auto".to_string(),
            background_attachment: "scroll".to_string(),
        }
    }
}

/// Group of reset flexbox properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetFlex {
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: i32,
    pub flex_direction: String,
    pub flex_wrap: String,
    pub justify_content: String,
    pub align_items: String,
    pub align_self: String,
    pub order: i32,
}

impl Default for ResetFlex {
    fn default() -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: -1,
            flex_direction: "row".to_string(),
            flex_wrap: "nowrap".to_string(),
            justify_content: "normal".to_string(),
            align_items: "normal".to_string(),
            align_self: "auto".to_string(),
            order: 0,
        }
    }
}

/// Group of reset table properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetTable {
    pub table_layout: String,
}

impl Default for ResetTable {
    fn default() -> Self {
        Self {
            table_layout: "auto".to_string(),
        }
    }
}

/// Group of reset visual effects and transitions properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetEffects {
    pub opacity: f32,
    pub outline_width: i32,
    pub outline_style: String,
    pub outline_color: String,
    pub transition_duration: u32,
    pub transition_property: String,
    pub text_decoration_line: String,
    pub text_decoration_color: String,
    pub text_decoration_style: String,
    pub text_overflow: String,
}

impl Default for ResetEffects {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            outline_width: -1,
            outline_style: "none".to_string(),
            outline_color: "invert".to_string(),
            transition_duration: 0,
            transition_property: "all".to_string(),
            text_decoration_line: "none".to_string(),
            text_decoration_color: "currentcolor".to_string(),
            text_decoration_style: "solid".to_string(),
            text_overflow: "clip".to_string(),
        }
    }
}

// Process-wide shared initial allocations for Style-Sharing.
static INITIAL_INHERITED_TEXT: OnceLock<Arc<InheritedText>> = OnceLock::new();
static INITIAL_INHERITED_LIST: OnceLock<Arc<InheritedList>> = OnceLock::new();
static INITIAL_INHERITED_TABLE: OnceLock<Arc<InheritedTable>> = OnceLock::new();
static INITIAL_INHERITED_UI: OnceLock<Arc<InheritedUI>> = OnceLock::new();
static INITIAL_INHERITED_EFFECTS: OnceLock<Arc<InheritedEffects>> = OnceLock::new();

static INITIAL_RESET_BOX: OnceLock<Arc<ResetBox>> = OnceLock::new();
static INITIAL_RESET_SURROUND: OnceLock<Arc<ResetSurround>> = OnceLock::new();
static INITIAL_RESET_BACKGROUND: OnceLock<Arc<ResetBackground>> = OnceLock::new();
static INITIAL_RESET_FLEX: OnceLock<Arc<ResetFlex>> = OnceLock::new();
static INITIAL_RESET_TABLE: OnceLock<Arc<ResetTable>> = OnceLock::new();
static INITIAL_RESET_EFFECTS: OnceLock<Arc<ResetEffects>> = OnceLock::new();

/// Production target style type per ADR 0001, carrying all 11 CSS categories.
#[derive(Debug, Clone, PartialEq)]
pub struct CategorizedComputedStyle {
    pub inherited_text: Arc<InheritedText>,
    pub inherited_list: Arc<InheritedList>,
    pub inherited_table: Arc<InheritedTable>,
    pub inherited_ui: Arc<InheritedUI>,
    pub inherited_effects: Arc<InheritedEffects>,
    pub reset_box: Arc<ResetBox>,
    pub reset_surround: Arc<ResetSurround>,
    pub reset_background: Arc<ResetBackground>,
    pub reset_flex: Arc<ResetFlex>,
    pub reset_table: Arc<ResetTable>,
    pub reset_effects: Arc<ResetEffects>,
}

impl CategorizedComputedStyle {
    /// Returns a new `CategorizedComputedStyle` sharing process-wide initial allocations.
    pub fn initial() -> Self {
        let inherited_text = INITIAL_INHERITED_TEXT
            .get_or_init(|| Arc::new(InheritedText::default()))
            .clone();
        let inherited_list = INITIAL_INHERITED_LIST
            .get_or_init(|| Arc::new(InheritedList::default()))
            .clone();
        let inherited_table = INITIAL_INHERITED_TABLE
            .get_or_init(|| Arc::new(InheritedTable::default()))
            .clone();
        let inherited_ui = INITIAL_INHERITED_UI
            .get_or_init(|| Arc::new(InheritedUI::default()))
            .clone();
        let inherited_effects = INITIAL_INHERITED_EFFECTS
            .get_or_init(|| Arc::new(InheritedEffects::default()))
            .clone();

        let reset_box = INITIAL_RESET_BOX
            .get_or_init(|| Arc::new(ResetBox::default()))
            .clone();
        let reset_surround = INITIAL_RESET_SURROUND
            .get_or_init(|| Arc::new(ResetSurround::default()))
            .clone();
        let reset_background = INITIAL_RESET_BACKGROUND
            .get_or_init(|| Arc::new(ResetBackground::default()))
            .clone();
        let reset_flex = INITIAL_RESET_FLEX
            .get_or_init(|| Arc::new(ResetFlex::default()))
            .clone();
        let reset_table = INITIAL_RESET_TABLE
            .get_or_init(|| Arc::new(ResetTable::default()))
            .clone();
        let reset_effects = INITIAL_RESET_EFFECTS
            .get_or_init(|| Arc::new(ResetEffects::default()))
            .clone();

        Self {
            inherited_text,
            inherited_list,
            inherited_table,
            inherited_ui,
            inherited_effects,
            reset_box,
            reset_surround,
            reset_background,
            reset_flex,
            reset_table,
            reset_effects,
        }
    }

    /// Inherits properties from a parent style node.
    /// Inherited categories are cloned (pointer copy, zero-alloc).
    /// Reset categories get the fresh process-wide initial allocation.
    pub fn inherit_from(parent: &Self) -> Self {
        let reset_box = INITIAL_RESET_BOX
            .get_or_init(|| Arc::new(ResetBox::default()))
            .clone();
        let reset_surround = INITIAL_RESET_SURROUND
            .get_or_init(|| Arc::new(ResetSurround::default()))
            .clone();
        let reset_background = INITIAL_RESET_BACKGROUND
            .get_or_init(|| Arc::new(ResetBackground::default()))
            .clone();
        let reset_flex = INITIAL_RESET_FLEX
            .get_or_init(|| Arc::new(ResetFlex::default()))
            .clone();
        let reset_table = INITIAL_RESET_TABLE
            .get_or_init(|| Arc::new(ResetTable::default()))
            .clone();
        let reset_effects = INITIAL_RESET_EFFECTS
            .get_or_init(|| Arc::new(ResetEffects::default()))
            .clone();

        Self {
            inherited_text: parent.inherited_text.clone(),
            inherited_list: parent.inherited_list.clone(),
            inherited_table: parent.inherited_table.clone(),
            inherited_ui: parent.inherited_ui.clone(),
            inherited_effects: parent.inherited_effects.clone(),
            reset_box,
            reset_surround,
            reset_background,
            reset_flex,
            reset_table,
            reset_effects,
        }
    }

    /// Set text color.
    pub fn set_color(&mut self, color: String) {
        Arc::make_mut(&mut self.inherited_text).color = color;
    }

    /// Set font size.
    pub fn set_font_size(&mut self, font_size: u32) {
        Arc::make_mut(&mut self.inherited_text).font_size = font_size;
    }

    /// Set display.
    pub fn set_display(&mut self, display: String) {
        Arc::make_mut(&mut self.reset_box).display = display;
    }

    /// Set width.
    pub fn set_width(&mut self, width: i32) {
        Arc::make_mut(&mut self.reset_box).width = width;
    }

    // TODO(spec): generated/typed setters for the full property set arrive with the cascade-migration task (item 2).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorized_style_sharing_and_inheritance() {
        let parent = CategorizedComputedStyle::initial();
        let child = CategorizedComputedStyle::inherit_from(&parent);

        // Inherited categories must have pointer equality between parent and child
        assert!(Arc::ptr_eq(&parent.inherited_text, &child.inherited_text));
        assert!(Arc::ptr_eq(&parent.inherited_list, &child.inherited_list));
        assert!(Arc::ptr_eq(&parent.inherited_table, &child.inherited_table));
        assert!(Arc::ptr_eq(&parent.inherited_ui, &child.inherited_ui));
        assert!(Arc::ptr_eq(
            &parent.inherited_effects,
            &child.inherited_effects
        ));

        // Reset categories must be shared with initial style (same shared pointers)
        let fresh_initial = CategorizedComputedStyle::initial();
        assert!(Arc::ptr_eq(&child.reset_box, &fresh_initial.reset_box));
        assert!(Arc::ptr_eq(
            &child.reset_surround,
            &fresh_initial.reset_surround
        ));
        assert!(Arc::ptr_eq(
            &child.reset_background,
            &fresh_initial.reset_background
        ));
        assert!(Arc::ptr_eq(&child.reset_flex, &fresh_initial.reset_flex));
        assert!(Arc::ptr_eq(&child.reset_table, &fresh_initial.reset_table));
        assert!(Arc::ptr_eq(
            &child.reset_effects,
            &fresh_initial.reset_effects
        ));
    }

    #[test]
    fn test_categorized_style_cow() {
        let original = CategorizedComputedStyle::initial();
        let mut cloned = original.clone();

        // Before any mutation, all category pointers must be equal
        assert!(Arc::ptr_eq(
            &original.inherited_text,
            &cloned.inherited_text
        ));
        assert!(Arc::ptr_eq(&original.reset_box, &cloned.reset_box));

        // Mutating cloned text color (inherited)
        cloned.set_color("red".to_string());

        // The mutated category should diverge
        assert_eq!(original.inherited_text.color, "black");
        assert_eq!(cloned.inherited_text.color, "red");
        assert!(!Arc::ptr_eq(
            &original.inherited_text,
            &cloned.inherited_text
        ));

        // Untouched categories must remain pointer-equal
        assert!(Arc::ptr_eq(&original.reset_box, &cloned.reset_box));
        assert!(Arc::ptr_eq(&original.reset_flex, &cloned.reset_flex));

        // Mutating cloned reset display
        cloned.set_display("block".to_string());
        assert_eq!(original.reset_box.display, "inline");
        assert_eq!(cloned.reset_box.display, "block");
        assert!(!Arc::ptr_eq(&original.reset_box, &cloned.reset_box));
    }

    #[test]
    fn test_initial_defaults() {
        let initial = CategorizedComputedStyle::initial();

        assert_eq!(initial.inherited_text.color, "black");
        assert_eq!(initial.inherited_text.font_size, 16);
        assert_eq!(initial.reset_box.display, "inline");
        assert_eq!(initial.reset_box.width, -1);
        assert_eq!(initial.reset_effects.opacity, 1.0);
    }
}
