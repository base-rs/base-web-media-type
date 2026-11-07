// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

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
