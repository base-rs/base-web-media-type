// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::{Error, MediaRange};
use std::fmt;
use std::str::FromStr;

/// A concrete media type, such as `text/plain; charset=utf-8`, which never contains a wildcard.
///
/// A media type is a [`MediaRange`] that names exactly one type, so it converts into one, and it can be used wherever a range is accepted through `AsRef<MediaRange>`. Converting a range into a media type fails when the range has a wildcard.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MediaType(MediaRange);

impl MediaType {
    /// `application/json`.
    pub const APPLICATION_JSON: Self = Self::from_static("application/json");
    /// `application/octet-stream`, for arbitrary binary data.
    pub const APPLICATION_OCTET_STREAM: Self = Self::from_static("application/octet-stream");
    /// `application/pdf`.
    pub const APPLICATION_PDF: Self = Self::from_static("application/pdf");
    /// `application/x-www-form-urlencoded`, for URL-encoded form content.
    pub const APPLICATION_X_WWW_FORM_URLENCODED: Self =
        Self::from_static("application/x-www-form-urlencoded");
    /// `application/xml`.
    pub const APPLICATION_XML: Self = Self::from_static("application/xml");
    /// `image/gif`.
    pub const IMAGE_GIF: Self = Self::from_static("image/gif");
    /// `image/jpeg`.
    pub const IMAGE_JPEG: Self = Self::from_static("image/jpeg");
    /// `image/png`.
    pub const IMAGE_PNG: Self = Self::from_static("image/png");
    /// `image/svg+xml`.
    pub const IMAGE_SVG_XML: Self = Self::from_static("image/svg+xml");
    /// `image/webp`.
    pub const IMAGE_WEBP: Self = Self::from_static("image/webp");
    /// `multipart/form-data`, for form content with files.
    pub const MULTIPART_FORM_DATA: Self = Self::from_static("multipart/form-data");
    /// `text/css`.
    pub const TEXT_CSS: Self = Self::from_static("text/css");
    /// `text/csv`.
    pub const TEXT_CSV: Self = Self::from_static("text/csv");
    /// `text/event-stream`, for server-sent events.
    pub const TEXT_EVENT_STREAM: Self = Self::from_static("text/event-stream");
    /// `text/html`.
    pub const TEXT_HTML: Self = Self::from_static("text/html");
    /// `text/javascript`.
    pub const TEXT_JAVASCRIPT: Self = Self::from_static("text/javascript");
    /// `text/markdown`.
    pub const TEXT_MARKDOWN: Self = Self::from_static("text/markdown");
    /// `text/plain`.
    pub const TEXT_PLAIN: Self = Self::from_static("text/plain");

    /// Parse a media type, such as `text/plain; charset=utf-8`, using the grammar of [`MediaRange::parse`]. Wildcards are rejected.
    pub fn parse(text: &str) -> Result<Self, Error> {
        MediaRange::parse(text)?.try_into()
    }

    /// A media type without parameters. Names must be tokens and are converted to lowercase; wildcards are rejected.
    pub fn new(top_level_type: &str, subtype: &str) -> Result<Self, Error> {
        MediaRange::new(top_level_type, subtype)?.try_into()
    }

    /// A media type from a lowercase static `type/subtype`, such as `application/json`, checked when evaluated, so a constant with an invalid name fails to compile.
    ///
    /// # Panics
    ///
    /// Panics if the name is not two lowercase tokens separated by one `/`, or if either is a wildcard.
    pub const fn from_static(essence: &'static str) -> Self {
        let range = MediaRange::from_static(essence);
        // Both names are tokens, so a wildcard component is a leading `*/` or a trailing `/*`.
        let bytes = essence.as_bytes();
        let length = bytes.len();
        assert!(
            !(bytes[0] == b'*' && bytes[1] == b'/')
                && !(bytes[length - 1] == b'*' && bytes[length - 2] == b'/'),
            "media types cannot contain wildcards"
        );
        Self(range)
    }

    /// Add a parameter, as with [`MediaRange::with_parameter`].
    pub fn with_parameter(self, name: &str, value: &str) -> Result<Self, Error> {
        Ok(Self(self.0.with_parameter(name, value)?))
    }

    /// The top-level type, such as `text`.
    pub fn top_level_type(&self) -> &str {
        self.0.top_level_type()
    }

    /// The subtype, such as `plain`.
    pub fn subtype(&self) -> &str {
        self.0.subtype()
    }

    /// The type and subtype without parameters, such as `text/plain`.
    pub fn essence(&self) -> &str {
        self.0.essence()
    }

    /// Parameter names and values, in order.
    pub fn parameters(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.0.parameters()
    }

    /// The value of a parameter. Names are compared case-insensitively.
    pub fn parameter(&self, name: &str) -> Option<&str> {
        self.0.parameter(name)
    }

    /// The same media type without parameters.
    pub fn without_parameters(&self) -> Self {
        Self(self.0.without_parameters())
    }

    /// Whether this media type is compatible with `range`, as with [`MediaRange::matches`].
    pub fn matches(&self, range: impl AsRef<MediaRange>) -> bool {
        self.0.matches(range)
    }

    /// This media type as a range.
    pub fn as_range(&self) -> &MediaRange {
        &self.0
    }
}

impl AsRef<MediaRange> for MediaType {
    fn as_ref(&self) -> &MediaRange {
        &self.0
    }
}

impl From<MediaType> for MediaRange {
    fn from(media_type: MediaType) -> Self {
        media_type.0
    }
}

impl TryFrom<MediaRange> for MediaType {
    type Error = Error;

    /// Fails with [`Error::Wildcard`] when the range has a wildcard type or subtype.
    fn try_from(range: MediaRange) -> Result<Self, Error> {
        if !range.is_concrete() {
            return Err(Error::Wildcard);
        }
        Ok(Self(range))
    }
}

impl FromStr for MediaType {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Error> {
        Self::parse(text)
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
