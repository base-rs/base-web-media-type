// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use base_web_media_type::{Error, MediaRange, MediaType};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn range(text: &str) -> MediaRange {
    MediaRange::parse(text).unwrap()
}

fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn parses_ranges_with_parameters() {
    let parsed = range("text/*; profile=\"example\"");
    assert_eq!(
        (parsed.top_level_type(), parsed.subtype(), parsed.essence()),
        ("text", "*", "text/*")
    );
    assert_eq!(
        parsed.parameters().collect::<Vec<_>>(),
        [("profile", "example")]
    );
    assert_eq!(parsed.to_string(), "text/*; profile=example");
    assert_eq!(
        "text/*".parse::<MediaRange>(),
        Ok(MediaRange::from_static("text/*"))
    );
    assert_eq!(range("*/*"), MediaRange::ANY);
}

#[test]
fn names_are_case_insensitive_and_values_keep_their_case() {
    let parsed = range("TEXT/Plain; CharSet=UTF-8");
    assert_eq!(parsed.essence(), "text/plain");
    assert_eq!(
        parsed.parameters().collect::<Vec<_>>(),
        [("charset", "UTF-8")]
    );
    assert_eq!(parsed.parameter("CHARSET"), Some("UTF-8"));
    assert_eq!(parsed.parameter("missing"), None);
    assert_eq!(
        MediaRange::new("TEXT", "PLAIN").unwrap().essence(),
        "text/plain"
    );
}

#[test]
fn follows_the_rfc_9110_grammar() {
    for valid in [
        "text/plain;",
        "text/plain; ",
        "text/plain;;charset=utf-8",
        "text/plain ;\tcharset=utf-8",
        "text/plain;charset=\"\"",
        "te*t/ht*l",
        "text/*+json",
    ] {
        assert!(MediaRange::parse(valid).is_ok(), "{valid}");
    }
    for invalid in [
        "",
        "text",
        "text/",
        "/plain",
        " text/plain",
        "text/plain ",
        "text/plain/extra",
        "text/plain; invalid",
        "text/plain; a",
        "text/plain; a=",
        "text/plain; a=b c",
        "text/plain; =b",
        "text plain",
        "text/plain; a=\"unterminated",
        "text/plain; a=\"trailing\\",
    ] {
        assert_eq!(
            MediaRange::parse(invalid),
            Err(Error::Syntax),
            "{invalid:?}"
        );
    }
    for control in [
        "text/plain; a=\"line\nbreak\"",
        "text/plain; a=\"bad\\\u{1}\"",
        "text/plain; a=\"del\u{7f}\"",
    ] {
        assert_eq!(
            MediaRange::parse(control),
            Err(Error::ParameterValue),
            "{control:?}"
        );
    }
    assert_eq!(MediaRange::parse("*/json"), Err(Error::Wildcard));
    assert_eq!(
        MediaRange::parse("text/plain; a=1; A=2"),
        Err(Error::DuplicateParameter)
    );
}

#[test]
fn quoted_values_are_unescaped_and_requoted() {
    let parsed = range(
        "text/plain; note=\"a,b; c\\\"d\"; delimiter=\",\"; unicode=\"caf\u{e9}\"; tab=\"a\tb\"; empty=\"\"",
    );
    assert_eq!(parsed.parameter("note"), Some("a,b; c\"d"));
    assert_eq!(parsed.parameter("unicode"), Some("caf\u{e9}"));
    assert_eq!(
        parsed.to_string(),
        "text/plain; note=\"a,b; c\\\"d\"; delimiter=\",\"; unicode=\"caf\u{e9}\"; tab=\"a\tb\"; empty=\"\""
    );
    assert_eq!(
        range(&parsed.to_string()),
        parsed,
        "serialization round-trips"
    );
    assert_eq!(
        range("text/plain; a=\"back\\\\slash\"").to_string(),
        "text/plain; a=\"back\\\\slash\""
    );
}

#[test]
fn builds_ranges_and_parameters() {
    let built = MediaRange::new("text", "*")
        .unwrap()
        .with_parameter("Format", "flowed")
        .unwrap();
    assert_eq!(built.to_string(), "text/*; format=flowed");
    assert_eq!(
        built.without_parameters(),
        MediaRange::from_static("text/*")
    );
    assert_eq!(MediaRange::new("*", "json"), Err(Error::Wildcard));
    assert_eq!(MediaRange::new("te xt", "plain"), Err(Error::Syntax));
    assert_eq!(MediaRange::new("text", ""), Err(Error::Syntax));
    assert_eq!(
        built.clone().with_parameter("bad name", "x"),
        Err(Error::Syntax)
    );
    assert_eq!(
        built.clone().with_parameter("format", "again"),
        Err(Error::DuplicateParameter)
    );
    assert_eq!(
        built.with_parameter("line", "a\nb"),
        Err(Error::ParameterValue)
    );
}

#[test]
fn static_ranges_are_checked() {
    for name in [
        "Text/plain",
        "text/",
        "/plain",
        "text",
        "te xt/plain",
        "a/b/c",
        "*/json",
    ] {
        assert!(
            std::panic::catch_unwind(|| MediaRange::from_static(name)).is_err(),
            "{name}"
        );
    }
    assert_eq!(MediaRange::from_static("text/*").essence(), "text/*");
}

#[test]
fn matching_uses_wildcards_and_ignores_parameters() {
    assert!(range("text/*").matches(MediaType::TEXT_PLAIN));
    assert!(range("*/*").matches(MediaType::APPLICATION_JSON));
    assert!(range("text/plain").matches(range("text/*")));
    assert!(range("text/plain").matches(MediaRange::ANY));
    assert!(range("application/json; version=2").matches(MediaType::APPLICATION_JSON));
    assert!(!range("text/*").matches(MediaType::APPLICATION_JSON));
    assert!(!range("text/plain").matches(MediaType::TEXT_HTML));
    assert!(!range("text/*").matches(range("application/*")));
    // A `*` inside a name is a literal character.
    assert!(range("te*t/ht*l").matches(MediaType::parse("te*t/ht*l").unwrap()));
    assert!(!range("te*t/ht*l").matches(MediaType::TEXT_HTML));
    assert!(range("text/plain").is_concrete());
    assert!(!range("text/*").is_concrete() && !MediaRange::ANY.is_concrete());
}

#[test]
fn equality_ignores_parameter_order() {
    let first = range("text/plain; a=1; b=2");
    let second = range("text/plain; b=2; a=1");
    assert_eq!(first, second);
    assert_eq!(hash(&first), hash(&second));
    assert_ne!(first, range("text/plain; a=1"));
    assert_ne!(first, range("text/plain; a=1; b=3"));
    assert_ne!(first, range("text/html; a=1; b=2"));
    assert_ne!(range("text/plain"), range("text/*"));
    assert_ne!(range("text/*"), range("application/*"));
}

/// A formatter destination that fails after accepting `remaining` characters.
struct Failing {
    remaining: usize,
}

impl std::fmt::Write for Failing {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        for _ in text.chars() {
            self.remaining = self.remaining.checked_sub(1).ok_or(std::fmt::Error)?;
        }
        Ok(())
    }
}

#[test]
fn formatting_errors_propagate() {
    use std::fmt::Write as _;
    let quoted = range("text/plain; a=\"q\\\"x\"");
    let length = quoted.to_string().chars().count();
    for remaining in 0..length {
        assert!(
            write!(Failing { remaining }, "{quoted}").is_err(),
            "{remaining}"
        );
    }
    assert!(write!(Failing { remaining: length }, "{quoted}").is_ok());
    assert!(write!(Failing { remaining: 3 }, "{}", MediaType::TEXT_PLAIN).is_err());
}

#[test]
fn the_number_of_parameters_is_limited() {
    let parameters = |count: usize| {
        (0..count)
            .map(|index| format!("; p{index}=v"))
            .collect::<String>()
    };
    let at_limit = range(&format!("a/b{}", parameters(MediaRange::MAX_PARAMETERS)));
    assert_eq!(at_limit.parameters().len(), MediaRange::MAX_PARAMETERS);
    assert_eq!(
        MediaRange::parse(&format!(
            "a/b{}",
            parameters(MediaRange::MAX_PARAMETERS + 1)
        )),
        Err(Error::TooManyParameters)
    );
    // A long input stops at the limit, so the quadratic duplicate check never grows past it.
    assert_eq!(
        MediaRange::parse(&format!("a/b{}", parameters(40_000))),
        Err(Error::TooManyParameters)
    );
    assert_eq!(
        at_limit.with_parameter("extra", "v"),
        Err(Error::TooManyParameters)
    );
}
