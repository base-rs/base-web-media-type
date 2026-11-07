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
//! assert_eq!(registry::for_path("site/index.html").map(|record| record.media_type()), Some(&MediaType::TEXT_HTML));
//! assert!(registry::get("Application/JSON").unwrap().extensions().contains(&"json"));
//! ```

use base_web_media_type::{MediaRange, MediaType};
use std::cmp::Ordering;
use std::path::Path;

#[rustfmt::skip]
mod records;

/// The `mime-types-data` release the tables were generated from.
pub const SOURCE_VERSION: &str = records::SOURCE_VERSION;

/// A registered media type and its associated data.
#[derive(Debug)]
pub struct Record {
    media_type: MediaType,
    encoding: ContentTransferEncoding,
    extensions: &'static [&'static str],
}

impl Record {
    const fn new(
        essence: &'static str,
        encoding: ContentTransferEncoding,
        extensions: &'static [&'static str],
    ) -> Self {
        Self {
            media_type: MediaType::from_static(essence),
            encoding,
            extensions,
        }
    }

    /// The media type, without parameters.
    pub fn media_type(&self) -> &MediaType {
        &self.media_type
    }

    /// The transfer encoding conventionally used for this type in MIME messages, such as email.
    pub fn encoding(&self) -> ContentTransferEncoding {
        self.encoding
    }

    /// Known filename extensions, in lowercase and without a leading dot, in the source data's order. Many types have none.
    pub fn extensions(&self) -> &'static [&'static str] {
        self.extensions
    }
}

/// A MIME content transfer encoding (RFC 2045).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ContentTransferEncoding {
    /// `7bit`: short lines of ASCII text.
    SevenBit,
    /// `8bit`: short lines of text that may use non-ASCII bytes.
    EightBit,
    /// `binary`: arbitrary bytes without line limits.
    Binary,
    /// `base64`, for binary data.
    Base64,
    /// `quoted-printable`, for mostly ASCII text.
    QuotedPrintable,
}

impl ContentTransferEncoding {
    /// The encoding's name in a `Content-Transfer-Encoding` field.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SevenBit => "7bit",
            Self::EightBit => "8bit",
            Self::Binary => "binary",
            Self::Base64 => "base64",
            Self::QuotedPrintable => "quoted-printable",
        }
    }
}

/// Every record, sorted by type and subtype.
pub fn records() -> &'static [Record] {
    &records::RECORDS
}

/// The record for a media type name, such as `application/json`. Parameters are not accepted; use [`for_media_type`] with a parsed type instead.
pub fn get(name: &str) -> Option<&'static Record> {
    let (top_level_type, subtype) = name.split_once('/')?;
    find(top_level_type, subtype)
}

/// The record for a media type or concrete range, ignoring its parameters.
pub fn for_media_type(media_type: impl AsRef<MediaRange>) -> Option<&'static Record> {
    let range = media_type.as_ref();
    find(range.top_level_type(), range.subtype())
}

/// The preferred record for a filename extension, with or without one leading dot, such as `json` or `.JSON`.
///
/// When several types list an extension, the one `mime-types-data` ranks as most common is chosen.
pub fn for_extension(extension: &str) -> Option<&'static Record> {
    let extension = extension.strip_prefix('.').unwrap_or(extension);
    let index = records::EXTENSIONS
        .binary_search_by(|(candidate, _)| compare_lowercase(candidate, extension))
        .ok()?;
    Some(&records::RECORDS[usize::from(records::EXTENSIONS[index].1)])
}

/// The preferred record for a path's final extension, such as `html` in `site/index.html`.
///
/// Compound extensions such as `warc.gz` are not considered; look them up with [`for_extension`].
pub fn for_path(path: impl AsRef<Path>) -> Option<&'static Record> {
    for_extension(path.as_ref().extension()?.to_str()?)
}

fn find(top_level_type: &str, subtype: &str) -> Option<&'static Record> {
    let index = records::RECORDS
        .binary_search_by(|record| {
            compare_lowercase(record.media_type.top_level_type(), top_level_type)
                .then_with(|| compare_lowercase(record.media_type.subtype(), subtype))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_can_be_built_at_runtime() {
        // Generated records are built at compile time, so build one at runtime for coverage of the constructor.
        let record = Record::new(
            "text/plain",
            ContentTransferEncoding::QuotedPrintable,
            &["txt"],
        );
        assert_eq!(
            (record.media_type(), record.encoding(), record.extensions()),
            (
                &MediaType::TEXT_PLAIN,
                ContentTransferEncoding::QuotedPrintable,
                &["txt"][..]
            )
        );
    }
}

#[cfg(doctest)]
#[doc = include_str!("../readme.md")]
struct Readme;
