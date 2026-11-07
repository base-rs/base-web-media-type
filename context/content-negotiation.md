# Content Negotiation

This guide explains how to combine `base-web-media-type` with an HTTP layer to select a representation.

## Responsibilities

Content negotiation has two parts:

- The HTTP layer parses the `Accept` field and orders its ranges by quality and preference.
- The application maps those ordered ranges to the representations it can produce.

This crate implements the second part only, so it does not depend on an HTTP implementation, and HTTP code does not depend on application policy.

## Register representations

Register the concrete types the application can produce. Registration order sets the default for wildcard ranges when there is no exact registration:

```rust
use base_web_media_type::{MediaType, MediaTypeMap};

let mut representations = MediaTypeMap::new();
representations.insert(MediaType::APPLICATION_JSON, "json");
representations.insert(MediaType::TEXT_PLAIN, "text");
```

Register concrete response types rather than client ranges. A server produces `application/json` even when a client asks for `application/*` or `*/*`; registering a wildcard is not possible.

## Select a representation

Pass the client's ranges in preference order to `select`. It returns the value for the first range with a compatible registration, together with that range, so the application can read its parameters:

```rust
use base_web_media_type::{MediaRange, MediaType, MediaTypeMap};

let representations: MediaTypeMap<&str> = [(MediaType::APPLICATION_JSON, "json"), (MediaType::TEXT_PLAIN, "text")].into_iter().collect();

// Parsed and ordered by the HTTP layer.
let accepted = [MediaRange::parse("image/*").unwrap(), MediaRange::parse("application/json; version=2").unwrap()];

let (representation, range) = representations.select(&accepted).expect("a compatible representation");
assert_eq!(*representation, "json");
assert_eq!(range.parameter("version"), Some("2"));
```

Parameters do not affect matching; they are inputs associated with the selected range.

## Missing headers and no match

HTTP treats a missing `Accept` field as accepting any type. Apply that policy at the HTTP boundary by selecting with `MediaRange::ANY`, which returns the first registration. When `select` returns `None`, an HTTP application normally responds with `406 Not Acceptable`.
