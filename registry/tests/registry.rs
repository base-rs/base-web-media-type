// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use base_web_media_type::{MediaRange, MediaType};
use base_web_media_type_registry::{self as registry, ContentTransferEncoding};

#[test]
fn looks_up_types_by_name_case_insensitively() {
    let record = registry::get("application/json").unwrap();
    assert_eq!(record.media_type(), &MediaType::APPLICATION_JSON);
    assert_eq!(record.encoding(), ContentTransferEncoding::EightBit);
    assert_eq!(record.extensions(), ["json", "map"]);
    assert!(std::ptr::eq(
        registry::get("APPLICATION/Json").unwrap(),
        record
    ));
    // Names with uppercase letters in the source data are stored in lowercase.
    assert!(registry::get("application/vnd.ms-excel.sheet.macroEnabled.12").is_some());
    for missing in [
        "application/unknown",
        "application",
        "",
        "application/json; charset=utf-8",
        "zzzz/zzzz",
    ] {
        assert!(registry::get(missing).is_none(), "{missing}");
    }
}

#[test]
fn looks_up_parsed_media_types() {
    let parsed = MediaType::parse("text/html; charset=utf-8").unwrap();
    assert_eq!(
        registry::for_media_type(&parsed).unwrap().media_type(),
        &MediaType::TEXT_HTML
    );
    assert!(registry::for_media_type(MediaRange::parse("text/*").unwrap()).is_none());
}

#[test]
fn looks_up_extensions_and_paths() {
    assert_eq!(
        registry::for_extension("json").unwrap().media_type(),
        &MediaType::APPLICATION_JSON
    );
    assert_eq!(
        registry::for_extension(".JSON").unwrap().media_type(),
        &MediaType::APPLICATION_JSON
    );
    assert_eq!(
        registry::for_extension("map").unwrap().media_type(),
        &MediaType::APPLICATION_JSON
    );
    assert_eq!(
        registry::for_extension("warc.gz")
            .unwrap()
            .media_type()
            .essence(),
        "application/warc+gz"
    );
    // Both cases of `Z` find the preferred compressed type.
    assert_eq!(
        registry::for_extension("Z").unwrap().media_type(),
        registry::for_extension("z").unwrap().media_type()
    );
    assert!(registry::for_extension("no-such-extension").is_none());
    assert!(registry::for_extension("").is_none());
    assert_eq!(
        registry::for_path("site/index.HTML").unwrap().media_type(),
        &MediaType::TEXT_HTML
    );
    assert_eq!(
        registry::for_path("archive.tar.gz")
            .unwrap()
            .media_type()
            .essence(),
        "application/gzip"
    );
    for none in ["README", ".bashrc", "directory/"] {
        assert!(registry::for_path(none).is_none(), "{none}");
    }
}

#[test]
fn common_extensions_map_to_their_usual_types() {
    let expected = [
        ("html", "text/html"),
        ("css", "text/css"),
        ("js", "text/javascript"),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("svg", "image/svg+xml"),
        ("pdf", "application/pdf"),
        ("txt", "text/plain"),
        ("wasm", "application/wasm"),
        ("woff2", "font/woff2"),
    ];
    for (extension, essence) in expected {
        assert_eq!(
            registry::for_extension(extension)
                .unwrap()
                .media_type()
                .essence(),
            essence,
            "{extension}"
        );
    }
}

#[test]
fn records_are_sorted_and_unique() {
    let records = registry::records();
    assert!(records.len() > 3000);
    let names: Vec<_> = records
        .iter()
        .map(|record| {
            (
                record.media_type().top_level_type(),
                record.media_type().subtype(),
            )
        })
        .collect();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(records.iter().all(|record| {
        record
            .extensions()
            .iter()
            .all(|extension| *extension == extension.to_ascii_lowercase())
    }));
    assert_eq!(registry::SOURCE_VERSION, "3.2026.1103");
}

#[test]
fn encodings_have_their_mime_names() {
    let names: Vec<_> = [
        ContentTransferEncoding::SevenBit,
        ContentTransferEncoding::EightBit,
        ContentTransferEncoding::Base64,
        ContentTransferEncoding::QuotedPrintable,
    ]
    .into_iter()
    .map(ContentTransferEncoding::as_str)
    .collect();
    assert_eq!(names, ["7bit", "8bit", "base64", "quoted-printable"]);
    assert_eq!(
        registry::get("text/plain").unwrap().encoding(),
        ContentTransferEncoding::QuotedPrintable
    );
    assert!(format!("{:?}", registry::get("text/plain").unwrap()).contains("plain"));
}

#[cfg(unix)]
#[test]
fn non_utf8_extensions_have_no_record() {
    use std::os::unix::ffi::OsStrExt;
    let path = std::path::Path::new(std::ffi::OsStr::from_bytes(b"file.\xff"));
    assert!(registry::for_path(path).is_none());
}
