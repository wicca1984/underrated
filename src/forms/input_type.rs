//! Typed HTML `<input>` element types and parser.

/// The parsed type of an HTML `<input>` element.
///
/// Covering the standard control types, with a fallback default of `Text`
/// as defined by the HTML Living Standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Checkbox,
    Radio,
    Submit,
    Reset,
    Button,
    Hidden,
    File,
    Email,
    Number,
    Search,
    Tel,
    Url,
    Color,
    Date,
    DateTimeLocal,
    Month,
    Range,
    Time,
    Week,
    Image,
}

impl InputType {
    /// Parses an HTML input `type` attribute string into an `InputType`.
    ///
    /// This matching is ASCII case-insensitive.
    /// Per the HTML spec, missing, empty, or unknown values default to `InputType::Text`.
    pub fn from_attr(s: &str) -> Self {
        match s {
            _ if s.eq_ignore_ascii_case("text") => InputType::Text,
            _ if s.eq_ignore_ascii_case("password") => InputType::Password,
            _ if s.eq_ignore_ascii_case("checkbox") => InputType::Checkbox,
            _ if s.eq_ignore_ascii_case("radio") => InputType::Radio,
            _ if s.eq_ignore_ascii_case("submit") => InputType::Submit,
            _ if s.eq_ignore_ascii_case("reset") => InputType::Reset,
            _ if s.eq_ignore_ascii_case("button") => InputType::Button,
            _ if s.eq_ignore_ascii_case("hidden") => InputType::Hidden,
            _ if s.eq_ignore_ascii_case("file") => InputType::File,
            _ if s.eq_ignore_ascii_case("email") => InputType::Email,
            _ if s.eq_ignore_ascii_case("number") => InputType::Number,
            _ if s.eq_ignore_ascii_case("search") => InputType::Search,
            _ if s.eq_ignore_ascii_case("tel") => InputType::Tel,
            _ if s.eq_ignore_ascii_case("url") => InputType::Url,
            _ if s.eq_ignore_ascii_case("color") => InputType::Color,
            _ if s.eq_ignore_ascii_case("date") => InputType::Date,
            _ if s.eq_ignore_ascii_case("datetime-local") => InputType::DateTimeLocal,
            _ if s.eq_ignore_ascii_case("month") => InputType::Month,
            _ if s.eq_ignore_ascii_case("range") => InputType::Range,
            _ if s.eq_ignore_ascii_case("time") => InputType::Time,
            _ if s.eq_ignore_ascii_case("week") => InputType::Week,
            _ if s.eq_ignore_ascii_case("image") => InputType::Image,
            _ => InputType::Text,
        }
    }
}

/// Helper function to parse an HTML `<input>` element's `type` attribute.
///
/// This matching is ASCII case-insensitive.
/// Per the HTML spec, missing, empty, or unknown values default to `InputType::Text`.
pub fn parse_input_type(s: &str) -> InputType {
    InputType::from_attr(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_keywords() {
        assert_eq!(InputType::from_attr("text"), InputType::Text);
        assert_eq!(InputType::from_attr("password"), InputType::Password);
        assert_eq!(InputType::from_attr("checkbox"), InputType::Checkbox);
        assert_eq!(InputType::from_attr("radio"), InputType::Radio);
        assert_eq!(InputType::from_attr("submit"), InputType::Submit);
        assert_eq!(InputType::from_attr("reset"), InputType::Reset);
        assert_eq!(InputType::from_attr("button"), InputType::Button);
        assert_eq!(InputType::from_attr("hidden"), InputType::Hidden);
        assert_eq!(InputType::from_attr("file"), InputType::File);
        assert_eq!(InputType::from_attr("email"), InputType::Email);
        assert_eq!(InputType::from_attr("number"), InputType::Number);
        assert_eq!(InputType::from_attr("search"), InputType::Search);
        assert_eq!(InputType::from_attr("tel"), InputType::Tel);
        assert_eq!(InputType::from_attr("url"), InputType::Url);
        assert_eq!(InputType::from_attr("color"), InputType::Color);
        assert_eq!(InputType::from_attr("date"), InputType::Date);
        assert_eq!(
            InputType::from_attr("datetime-local"),
            InputType::DateTimeLocal
        );
        assert_eq!(InputType::from_attr("month"), InputType::Month);
        assert_eq!(InputType::from_attr("range"), InputType::Range);
        assert_eq!(InputType::from_attr("time"), InputType::Time);
        assert_eq!(InputType::from_attr("week"), InputType::Week);
        assert_eq!(InputType::from_attr("image"), InputType::Image);
    }

    #[test]
    fn test_mixed_case_matching() {
        assert_eq!(InputType::from_attr("CheckBox"), InputType::Checkbox);
        assert_eq!(InputType::from_attr("RADIO"), InputType::Radio);
        assert_eq!(InputType::from_attr("SubMit"), InputType::Submit);
        assert_eq!(InputType::from_attr("PassWord"), InputType::Password);
    }

    #[test]
    fn test_unknown_and_empty_fallback() {
        assert_eq!(InputType::from_attr("bogus"), InputType::Text);
        assert_eq!(InputType::from_attr(""), InputType::Text);
        assert_eq!(
            InputType::from_attr("unknown-type-attribute"),
            InputType::Text
        );
    }

    #[test]
    fn test_default_impl() {
        assert_eq!(InputType::default(), InputType::Text);
    }

    #[test]
    fn test_parse_input_type_helper() {
        assert_eq!(parse_input_type("radio"), InputType::Radio);
        assert_eq!(parse_input_type("invalid-type"), InputType::Text);
    }
}
