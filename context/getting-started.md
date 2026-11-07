# Getting Started

This guide explains how to represent, compare, and match media types with `base-web-media-type`.

## Media types and ranges

A `MediaType` names one kind of content, such as `text/html; charset=utf-8`. A `MediaRange` can also use wildcards, such as `text/*` or `*/*`, as in an HTTP `Accept` field. Parse them from text, or build them from names:

```rust
use base_web_media_type::{MediaRange, MediaType};

let html = MediaType::parse("text/HTML; charset=utf-8").unwrap();
assert_eq!(html.essence(), "text/html");
assert_eq!(html.parameter("charset"), Some("utf-8"));

let text = MediaRange::new("text", "*").unwrap();
assert!(text.matches(&html));

let json = MediaType::APPLICATION_JSON.with_parameter("version", "2").unwrap();
assert_eq!(json.to_string(), "application/json; version=2");
```

Names are case-insensitive and stored in lowercase; parameter values keep their case. Parsing follows RFC 9110 exactly and does not trim whitespace, so trim field values first. Parsing a wildcard as a `MediaType` fails with `Error::Wildcard`.

Constants such as `MediaType::TEXT_PLAIN` cover common types. Define others with `MediaType::from_static`, which checks its names when the constant is evaluated:

```rust
use base_web_media_type::MediaType;

const ATOM: MediaType = MediaType::from_static("application/atom+xml");
```

A `MediaType` converts into a `MediaRange`, and every function that takes a range accepts either through `AsRef<MediaRange>`.

## Matching and equality

`matches` compares types and subtypes, treating a wildcard on either side as compatible, and ignores parameters. Equality compares parameters too, ignoring their order:

```rust
use base_web_media_type::{MediaRange, MediaType};

let versioned = MediaType::parse("application/json; version=2").unwrap();
assert!(versioned.matches(MediaType::APPLICATION_JSON));
assert_ne!(versioned, MediaType::APPLICATION_JSON);
assert!(MediaRange::ANY.matches(&versioned));
```

## Sets

A `MediaRangeSet` holds the types something accepts, such as an upload field. Check a concrete type with `contains`:

```rust
use base_web_media_type::{MediaRange, MediaRangeSet, MediaType};

let allowed: MediaRangeSet = [MediaRange::new("image", "*").unwrap(), MediaType::APPLICATION_PDF.into()].into_iter().collect();
assert!(allowed.contains(&MediaType::IMAGE_PNG));
assert!(!allowed.contains(&MediaType::TEXT_HTML));
```

Parse an upload's `Content-Type` as a `MediaType`, never a `MediaRange`, so a client cannot send a wildcard such as `*/*`. `matches` instead asks whether any member is compatible with a range, so `*/*` matches any non-empty set. Ranges covered by a wildcard are absorbed, and adding `*/*` makes the set universal.

## Maps

A `MediaTypeMap` associates concrete types with values and finds the value for a range. Registration order sets the defaults for wildcard queries; see [Content Negotiation](content-negotiation.md).

## Registry

The `base-web-media-type-registry` crate maps names and filename extensions to registered types; see [Registry](registry.md).
