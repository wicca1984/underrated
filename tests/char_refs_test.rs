use underrated::encoding::InputStream;
use underrated::html::{Token, Tokenizer};

#[test]
fn test_numeric_entities() {
    let inputs = [("&#65;", 'A'), ("&#x41;", 'A'), ("&#X41;", 'A')];
    for (input, expected) in inputs {
        let stream = InputStream::from_utf8(input.as_bytes());
        let mut tokenizer = Tokenizer::new(stream);
        assert_eq!(
            tokenizer.next_token(),
            Token::Character(expected),
            "Failed for {}",
            input
        );
    }
}

#[test]
fn test_numeric_entities_no_normalization() {
    let inputs = [
        ("&#13;", '\r'),
        ("&#x0D;", '\r'),
        ("&#128;", '\u{20AC}'), // EURO SIGN
        ("&#x9F;", '\u{0178}'), // LATIN CAPITAL LETTER Y WITH DIAERESIS
    ];
    for (input, expected) in inputs {
        let stream = InputStream::from_utf8(input.as_bytes());
        let mut tokenizer = Tokenizer::new(stream);
        assert_eq!(
            tokenizer.next_token(),
            Token::Character(expected),
            "Failed for {}",
            input
        );
    }
}

#[test]
fn test_named_entities() {
    let inputs = [
        ("&amp;", '&'),
        ("&amp", '&'),
        ("&lt;", '<'),
        ("&lt", '<'),
        ("&gt;", '>'),
        ("&gt", '>'),
        ("&quot;", '"'),
        ("&quot", '"'),
        ("&apos;", '\''),
        // `apos`/`trade` are NOT in the legacy no-semicolon entity set, so without a
        // trailing `;` they are emitted literally (html5lib namedEntities.test:
        // "Bad named entity: apos without a semi-colon"). The no-semicolon literal
        // forms are covered by test_no_semicolon_literal below.
        ("&nbsp;", '\u{00A0}'),
        ("&nbsp", '\u{00A0}'),
        ("&copy;", '©'),
        ("&copy", '©'),
        ("&reg;", '®'),
        ("&reg", '®'),
        ("&trade;", '™'),
        ("&deg;", '°'),
        ("&deg", '°'),
        ("&plusmn;", '±'),
        ("&plusmn", '±'),
    ];
    for (input, expected) in inputs {
        let stream = InputStream::from_utf8(input.as_bytes());
        let mut tokenizer = Tokenizer::new(stream);
        assert_eq!(
            tokenizer.next_token(),
            Token::Character(expected),
            "Failed for {}",
            input
        );
    }
}

#[test]
fn test_no_semicolon_literal() {
    // Non-legacy named references without a trailing `;` are not decoded; they are
    // emitted literally (html5lib namedEntities.test "Bad named entity ... without a
    // semi-colon"). This is the spec-correct counterpart to the legacy cases above.
    let inputs = [("&apos", "&apos"), ("&trade", "&trade")];
    for (input, expected) in inputs {
        let stream = InputStream::from_utf8(input.as_bytes());
        let mut tokenizer = Tokenizer::new(stream);
        let mut actual = String::new();
        loop {
            match tokenizer.next_token() {
                Token::Character(c) => actual.push(c),
                Token::Eof => break,
                other => panic!("Unexpected token {:?} for {}", other, input),
            }
        }
        assert_eq!(actual, expected, "Failed for {}", input);
    }
}

#[test]
fn test_invalid_entities() {
    let inputs = [
        ("&unknown;", "&unknown;"),
        ("&#;", "&#;"),
        ("&#x;", "&#x;"),
        ("&noti;", "¬i;"), // semicolonless &not matches first, leaving "i;"
    ];
    for (input, expected) in inputs {
        let stream = InputStream::from_utf8(input.as_bytes());
        let mut tokenizer = Tokenizer::new(stream);
        let mut actual = String::new();
        loop {
            match tokenizer.next_token() {
                Token::Character(c) => actual.push(c),
                Token::Eof => break,
                _ => panic!("Unexpected token"),
            }
        }
        assert_eq!(actual, expected, "Failed for {}", input);
    }
}

#[test]
fn test_entities_in_attributes() {
    let input = "<div a=\"&amp;\" b='&#65;' c='&lt;'>";
    let stream = InputStream::from_utf8(input.as_bytes());
    let mut tokenizer = Tokenizer::new(stream);
    let tok = tokenizer.next_token();
    if let Token::StartTag { mut attrs, .. } = tok {
        attrs.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            attrs,
            vec![
                ("a".to_string(), "&".to_string()),
                ("b".to_string(), "A".to_string()),
                ("c".to_string(), "<".to_string()),
            ]
        );
    } else {
        panic!("Expected StartTag, got {:?}", tok);
    }
}
