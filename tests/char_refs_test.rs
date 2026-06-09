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
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&quot;", '"'),
        ("&apos;", '\''),
        ("&nbsp;", '\u{00A0}'),
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
fn test_invalid_entities() {
    let inputs = [("&unknown;", "&unknown;"), ("&#;", "&#;"), ("&#x;", "&#x;")];
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
