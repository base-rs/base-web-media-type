// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::{Error, syntax};
use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

const WILDCARD: &str = "*";

/// A media range, such as `text/*` or `*/*`, which may use wildcards, or a concrete media type, such as `text/plain`, with its parameters.
///
/// Type and subtype names and parameter names are case-insensitive, so they are stored in lowercase. Parameter values keep their case and order. A wildcard is a component that is exactly `*`; a `*` inside a name, as in `text/*+json`, is an ordinary token character.
///
/// Equality compares names and parameters, ignoring parameter order. Matching with [`Self::matches`] ignores parameters.
#[derive(Clone, Debug)]
pub struct MediaRange {
    /// `type/subtype` in lowercase.
    essence: Cow<'static, str>,
    /// The position of the `/` in `essence`.
    slash: usize,
    parameters: Vec<(String, String)>,
}

impl MediaRange {
    /// The range that matches every media type, `*/*`.
    pub const ANY: Self = Self::from_static("*/*");

    /// The most parameters a range can have. Real media types have a handful; the limit bounds the work that parsing, comparing, and hashing untrusted input can cause.
    pub const MAX_PARAMETERS: usize = 64;

    /// Parse a media range, such as `text/*; charset=utf-8`.
    ///
    /// The text must follow RFC 9110's `media-type` grammar exactly, without surrounding whitespace. Parameter values may be tokens or quoted strings, and empty parameters such as a trailing `;` are permitted. A wildcard type requires a wildcard subtype, a parameter name may appear only once, and there may be at most [`Self::MAX_PARAMETERS`] parameters.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let parsed = syntax::parse(text)?;
        let range = Self {
            essence: Cow::Owned(parsed.essence),
            slash: parsed.slash,
            parameters: parsed.parameters,
        };
        range.check_wildcards()?;
        Ok(range)
    }

    /// A media range without parameters. Names must be tokens and are converted to lowercase.
    pub fn new(top_level_type: &str, subtype: &str) -> Result<Self, Error> {
        if !syntax::is_token(top_level_type) || !syntax::is_token(subtype) {
            return Err(Error::Syntax);
        }
        let range = Self::from_parts(
            &top_level_type.to_ascii_lowercase(),
            &subtype.to_ascii_lowercase(),
        );
        range.check_wildcards()?;
        Ok(range)
    }

    /// A range from a lowercase static `type/subtype`, such as `text/*`, checked when evaluated, so a constant with an invalid name fails to compile.
    ///
    /// # Panics
    ///
    /// Panics if the name is not two lowercase tokens separated by one `/`, or if a wildcard type has a concrete subtype.
    pub const fn from_static(essence: &'static str) -> Self {
        let bytes = essence.as_bytes();
        let mut slash = 0;
        while slash < bytes.len() && bytes[slash] != b'/' {
            slash += 1;
        }
        assert!(
            slash < bytes.len(),
            "a media range name needs a type and a subtype separated by `/`"
        );
        let (top_level_type, subtype) = bytes.split_at(slash);
        let subtype = subtype.split_at(1).1;
        assert!(
            is_static_name(top_level_type) && is_static_name(subtype),
            "media range names must be lowercase tokens"
        );
        assert!(
            !is_wildcard(top_level_type) || is_wildcard(subtype),
            "a wildcard type requires a wildcard subtype"
        );
        Self {
            essence: Cow::Borrowed(essence),
            slash,
            parameters: Vec::new(),
        }
    }

    /// Add a parameter. The name must be a token and is converted to lowercase; the value can be any text without control characters other than tab.
    pub fn with_parameter(mut self, name: &str, value: &str) -> Result<Self, Error> {
        if !syntax::is_token(name) {
            return Err(Error::Syntax);
        }
        if !value.chars().all(syntax::is_quotable) {
            return Err(Error::ParameterValue);
        }
        let name = name.to_ascii_lowercase();
        if self
            .parameters
            .iter()
            .any(|(existing, _)| *existing == name)
        {
            return Err(Error::DuplicateParameter);
        }
        if self.parameters.len() == Self::MAX_PARAMETERS {
            return Err(Error::TooManyParameters);
        }
        self.parameters.push((name, value.to_owned()));
        Ok(self)
    }

    /// The top-level type, such as `text`, or `*`.
    pub fn top_level_type(&self) -> &str {
        &self.essence[..self.slash]
    }

    /// The subtype, such as `plain`, or `*`.
    pub fn subtype(&self) -> &str {
        &self.essence[self.slash + 1..]
    }

    /// The type and subtype without parameters, such as `text/plain`.
    pub fn essence(&self) -> &str {
        &self.essence
    }

    /// Parameter names and values, in order.
    pub fn parameters(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.parameters
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// The value of a parameter. Names are compared case-insensitively.
    pub fn parameter(&self, name: &str) -> Option<&str> {
        self.parameters
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The same range without parameters.
    pub fn without_parameters(&self) -> Self {
        Self {
            essence: self.essence.clone(),
            slash: self.slash,
            parameters: Vec::new(),
        }
    }

    /// Whether neither the type nor the subtype is a wildcard.
    pub fn is_concrete(&self) -> bool {
        !self.is_wildcard_type() && !self.is_wildcard_subtype()
    }

    /// Whether the type and subtype are compatible with `other`'s. A wildcard on either side matches any name; parameters are ignored.
    pub fn matches(&self, other: impl AsRef<MediaRange>) -> bool {
        let other = other.as_ref();
        component_matches(self.top_level_type(), other.top_level_type())
            && component_matches(self.subtype(), other.subtype())
    }

    /// A range from names that are already validated and lowercase.
    pub(crate) fn from_parts(top_level_type: &str, subtype: &str) -> Self {
        Self {
            essence: Cow::Owned(format!("{top_level_type}/{subtype}")),
            slash: top_level_type.len(),
            parameters: Vec::new(),
        }
    }

    pub(crate) fn is_wildcard_type(&self) -> bool {
        self.top_level_type() == WILDCARD
    }

    pub(crate) fn is_wildcard_subtype(&self) -> bool {
        self.subtype() == WILDCARD
    }

    fn check_wildcards(&self) -> Result<(), Error> {
        if self.is_wildcard_type() && !self.is_wildcard_subtype() {
            return Err(Error::Wildcard);
        }
        Ok(())
    }
}

fn component_matches(component: &str, other: &str) -> bool {
    component == WILDCARD || other == WILDCARD || component == other
}

const fn is_wildcard(name: &[u8]) -> bool {
    matches!(name, [b'*'])
}

const fn is_static_name(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        if !syntax::is_token_byte(bytes[index]) || bytes[index].is_ascii_uppercase() {
            return false;
        }
        index += 1;
    }
    true
}

impl AsRef<MediaRange> for MediaRange {
    fn as_ref(&self) -> &MediaRange {
        self
    }
}

impl FromStr for MediaRange {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Error> {
        Self::parse(text)
    }
}

impl fmt::Display for MediaRange {
    /// Writes `type/subtype` followed by `; name=value` for each parameter, quoting values that are not tokens.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.essence)?;
        for (name, value) in &self.parameters {
            write!(formatter, "; {name}=")?;
            syntax::write_value(formatter, value)?;
        }
        Ok(())
    }
}

impl PartialEq for MediaRange {
    fn eq(&self, other: &Self) -> bool {
        // Parameter names are unique and there are at most `MAX_PARAMETERS`, so this comparison is bounded.
        self.essence == other.essence
            && self.parameters.len() == other.parameters.len()
            && self
                .parameters
                .iter()
                .all(|parameter| other.parameters.contains(parameter))
    }
}

impl Eq for MediaRange {}

impl Hash for MediaRange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.essence.hash(state);
        // Parameter order does not affect equality, so hash the parameters in a canonical order.
        let mut parameters: Vec<_> = self.parameters.iter().collect();
        parameters.sort_unstable();
        parameters.hash(state);
    }
}
