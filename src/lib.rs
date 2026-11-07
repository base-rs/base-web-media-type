// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! Media types and media ranges, with sets and maps that match by range.
//!
//! A [`MediaType`] names one kind of content, such as `text/html; charset=utf-8`. A [`MediaRange`] can also use wildcards, such as `text/*` or `*/*`, as in an HTTP `Accept` field. Both follow the grammar of RFC 9110 and normalize case-insensitive names to lowercase.
//!
//! [`MediaRangeSet`] answers whether a type or range is acceptable, and [`MediaTypeMap`] finds a value, such as a renderer, for the first acceptable range. Both ignore parameters when matching.
//!
//! ```
//! use base_web_media_type::{MediaRange, MediaType};
//!
//! let media_type = MediaType::parse("text/HTML; charset=utf-8")?;
//! assert_eq!(media_type.essence(), "text/html");
//! assert_eq!(media_type.parameter("Charset"), Some("utf-8"));
//! assert!(MediaRange::parse("text/*")?.matches(&media_type));
//! # Ok::<(), base_web_media_type::MediaTypeError>(())
//! ```
//!
//! This crate does not parse HTTP fields such as `Accept`, or order ranges by quality. Those belong to the HTTP layer, which can produce [`MediaRange`] values for these types to match.

mod media_range;
mod media_range_set;
mod media_type;
mod media_type_error;
mod media_type_map;
mod syntax;

pub use media_range::MediaRange;
pub use media_range_set::MediaRangeSet;
pub use media_type::MediaType;
pub use media_type_error::MediaTypeError;
pub use media_type_map::{MediaTypeMap, MediaTypeMapIter};

#[cfg(doctest)]
#[doc = include_str!("../context/getting-started.md")]
struct GettingStartedGuide;

#[cfg(doctest)]
#[doc = include_str!("../context/content-negotiation.md")]
struct ContentNegotiationGuide;

#[cfg(doctest)]
#[doc = include_str!("../readme.md")]
struct Readme;
