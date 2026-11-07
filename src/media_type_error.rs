// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use std::fmt;

/// Why a media type, media range, or parameter was rejected.
///
/// Errors do not repeat the rejected input, which may be large or untrusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaTypeError {
    /// The text does not follow the `type/subtype *( OWS ";" OWS [ parameter ] )` grammar of RFC 9110, or a name is not a token.
    Syntax,
    /// A media type contains a wildcard, or a media range uses a wildcard type with a concrete subtype, such as `*/json`.
    Wildcard,
    /// A parameter name appears more than once. Names are compared case-insensitively.
    DuplicateParameter,
    /// A parameter value contains a control character other than horizontal tab, so it cannot be written as a quoted string.
    ParameterValue,
    /// A media range has more than [`MediaRange::PARAMETER_COUNT_LIMIT`](crate::MediaRange::PARAMETER_COUNT_LIMIT) parameters.
    TooManyParameters,
}

impl fmt::Display for MediaTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Syntax => "invalid media type syntax",
            Self::Wildcard => "invalid wildcard in media type",
            Self::DuplicateParameter => "duplicate media type parameter",
            Self::ParameterValue => "media type parameter value contains a control character",
            Self::TooManyParameters => "too many media type parameters",
        })
    }
}

impl std::error::Error for MediaTypeError {}
