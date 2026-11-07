// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::ContentTransferEncoding;
use base_web_media_type::MediaType;

/// A registered media type and its associated data.
#[derive(Debug)]
pub struct MediaTypeRecord {
    media_type: MediaType,
    encoding: ContentTransferEncoding,
    extensions: &'static [&'static str],
}

impl MediaTypeRecord {
    /// A record for the generated tables, checked when evaluated.
    pub(crate) const fn new(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_can_be_built_at_runtime() {
        // Generated records are built at compile time, so build one at runtime for coverage of the constructor.
        let record = MediaTypeRecord::new(
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
