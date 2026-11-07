// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! Registered media types, with their conventional encodings and filename extensions.
//!
//! The data is generated from the [`mime-types-data`](https://github.com/mime-types/mime-types-data) project, which combines the IANA media type registry with types and extensions in common use. Lookups use static tables and do not allocate. Names and extensions are matched case-insensitively.
//!
//! ```
//! use base_web_media_type::MediaType;
//! use base_web_media_type_registry as registry;
//!
//! assert_eq!(registry::by_path("site/index.html").map(|record| record.media_type()), Some(&MediaType::TEXT_HTML));
//! assert!(registry::by_name("Application/JSON").unwrap().extensions().contains(&"json"));
//! ```

use base_web_media_type::MediaRange;
use std::cmp::Ordering;
use std::path::Path;

mod content_transfer_encoding;
mod media_type_record;
#[rustfmt::skip]
mod records;

pub use content_transfer_encoding::ContentTransferEncoding;
pub use media_type_record::MediaTypeRecord;

/// The `mime-types-data` release the tables were generated from.
pub const SOURCE_VERSION: &str = records::SOURCE_VERSION;

/// Every record, sorted by type and then subtype.
pub fn records() -> &'static [MediaTypeRecord] {
    &records::RECORDS
}

/// The record for a media type name, such as `application/json`. Parameters are not accepted; use [`by_media_type`] with a parsed type instead.
pub fn by_name(name: &str) -> Option<&'static MediaTypeRecord> {
    let (top_level_type, subtype) = name.split_once('/')?;
    find(top_level_type, subtype)
}

/// The record for a media type or concrete range, ignoring its parameters.
pub fn by_media_type(media_type: impl AsRef<MediaRange>) -> Option<&'static MediaTypeRecord> {
    let range = media_type.as_ref();
    find(range.top_level_type(), range.subtype())
}

/// The preferred record for a filename extension, with or without one leading dot, such as `json` or `.JSON`.
///
/// When several types list an extension, the one `mime-types-data` ranks as most common is chosen.
pub fn by_extension(extension: &str) -> Option<&'static MediaTypeRecord> {
    let extension = extension.strip_prefix('.').unwrap_or(extension);
    let index = records::EXTENSIONS
        .binary_search_by(|(candidate, _)| compare_lowercase(candidate, extension))
        .ok()?;
    Some(&records::RECORDS[usize::from(records::EXTENSIONS[index].1)])
}

/// The preferred record for a path's final extension, such as `html` in `site/index.html`.
///
/// Compound extensions such as `warc.gz` are not considered; look them up with [`by_extension`].
pub fn by_path(path: impl AsRef<Path>) -> Option<&'static MediaTypeRecord> {
    by_extension(path.as_ref().extension()?.to_str()?)
}

fn find(top_level_type: &str, subtype: &str) -> Option<&'static MediaTypeRecord> {
    let index = records::RECORDS
        .binary_search_by(|record| {
            compare_lowercase(record.media_type().top_level_type(), top_level_type)
                .then_with(|| compare_lowercase(record.media_type().subtype(), subtype))
        })
        .ok()?;
    Some(&records::RECORDS[index])
}

/// Compare a lowercase table entry with a query of any case, as if the query were lowercase.
fn compare_lowercase(entry: &str, query: &str) -> Ordering {
    entry
        .bytes()
        .cmp(query.bytes().map(|byte| byte.to_ascii_lowercase()))
}

#[cfg(doctest)]
#[doc = include_str!("../readme.md")]
struct Readme;
