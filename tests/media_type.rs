// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use base_web_media_type::{Error, MediaRange, MediaType};

#[test]
fn parses_concrete_media_types() {
    let media_type = MediaType::parse("text/plain; charset=utf-8; title=\"Plain Text\"").unwrap();
    assert_eq!(
        (
            media_type.top_level_type(),
            media_type.subtype(),
            media_type.essence()
        ),
        ("text", "plain", "text/plain")
    );
    assert_eq!(
        media_type.parameters().collect::<Vec<_>>(),
        [("charset", "utf-8"), ("title", "Plain Text")]
    );
    assert_eq!(media_type.parameter("TITLE"), Some("Plain Text"));
    assert_eq!(
        media_type.to_string(),
        "text/plain; charset=utf-8; title=\"Plain Text\""
    );
    assert_eq!(media_type.without_parameters(), MediaType::TEXT_PLAIN);
    assert_eq!("TEXT/PLAIN".parse::<MediaType>(), Ok(MediaType::TEXT_PLAIN));
}

#[test]
fn wildcards_and_invalid_names_are_rejected() {
    for wildcard in ["text/*", "*/*"] {
        assert_eq!(
            MediaType::parse(wildcard),
            Err(Error::Wildcard),
            "{wildcard}"
        );
    }
    for invalid in ["text", "text/plain/extra", "text/plain; invalid"] {
        assert_eq!(MediaType::parse(invalid), Err(Error::Syntax), "{invalid}");
    }
    assert_eq!(MediaType::new("text", "*"), Err(Error::Wildcard));
    assert_eq!(MediaType::new("te xt", "plain"), Err(Error::Syntax));
    assert_eq!(MediaType::new("TEXT", "PLAIN"), Ok(MediaType::TEXT_PLAIN));
    assert_eq!(
        MediaType::TEXT_PLAIN.with_parameter("charset", "a\u{7f}"),
        Err(Error::ParameterValue)
    );
    for name in ["*/*", "text/*", "Text/plain", "text"] {
        assert!(
            std::panic::catch_unwind(|| MediaType::from_static(name)).is_err(),
            "{name}"
        );
    }
}

#[test]
fn media_types_are_ranges() {
    let media_type = MediaType::TEXT_HTML
        .with_parameter("charset", "utf-8")
        .unwrap();
    assert!(media_type.matches(MediaRange::parse("text/*").unwrap()));
    assert!(!media_type.matches(MediaType::TEXT_PLAIN));
    assert_eq!(
        media_type.as_range(),
        AsRef::<MediaRange>::as_ref(&media_type)
    );
    let range: MediaRange = media_type.clone().into();
    assert_eq!(range.to_string(), "text/html; charset=utf-8");
    assert_eq!(MediaType::try_from(range), Ok(media_type));
    assert_eq!(MediaType::try_from(MediaRange::ANY), Err(Error::Wildcard));
}

#[test]
fn constants_are_valid_media_types() {
    let constants = [
        (MediaType::APPLICATION_JSON, "application/json"),
        (
            MediaType::APPLICATION_OCTET_STREAM,
            "application/octet-stream",
        ),
        (MediaType::APPLICATION_PDF, "application/pdf"),
        (
            MediaType::APPLICATION_X_WWW_FORM_URLENCODED,
            "application/x-www-form-urlencoded",
        ),
        (MediaType::APPLICATION_XML, "application/xml"),
        (MediaType::IMAGE_GIF, "image/gif"),
        (MediaType::IMAGE_JPEG, "image/jpeg"),
        (MediaType::IMAGE_PNG, "image/png"),
        (MediaType::IMAGE_SVG_XML, "image/svg+xml"),
        (MediaType::IMAGE_WEBP, "image/webp"),
        (MediaType::MULTIPART_FORM_DATA, "multipart/form-data"),
        (MediaType::TEXT_CSS, "text/css"),
        (MediaType::TEXT_CSV, "text/csv"),
        (MediaType::TEXT_EVENT_STREAM, "text/event-stream"),
        (MediaType::TEXT_HTML, "text/html"),
        (MediaType::TEXT_JAVASCRIPT, "text/javascript"),
        (MediaType::TEXT_MARKDOWN, "text/markdown"),
        (MediaType::TEXT_PLAIN, "text/plain"),
    ];
    for (constant, text) in constants {
        assert_eq!(constant, MediaType::parse(text).unwrap());
    }
}

#[test]
fn errors_describe_the_problem() {
    let messages: Vec<String> = [
        Error::Syntax,
        Error::Wildcard,
        Error::DuplicateParameter,
        Error::ParameterValue,
        Error::TooManyParameters,
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    assert_eq!(
        messages,
        [
            "invalid media type syntax",
            "invalid wildcard in media type",
            "duplicate media type parameter",
            "media type parameter value contains a control character",
            "too many media type parameters"
        ]
    );
    let error: Box<dyn std::error::Error> = Box::new(Error::Syntax);
    assert!(error.source().is_none());
}
