# Media Types and Ranges

## Requirement

Concrete media types and media ranges must be distinct types. A `MediaType` never contains a wildcard; a `MediaRange` may use `*` as a whole type or subtype, and a wildcard type requires a wildcard subtype. A `*` inside a name is an ordinary token character.

Parsing must follow the `media-type` grammar of RFC 9110 (sections 5.6 and 8.3.1) without accepting surrounding whitespace: tokens for names, token or quoted-string parameter values, optional whitespace around semicolons, and empty parameters. Type, subtype, and parameter names are case-insensitive and normalized to lowercase; parameter values keep their case and order. A repeated parameter name, more than 64 parameters (`MediaRange::PARAMETER_COUNT_LIMIT`), a quoted value containing a control character other than horizontal tab, and any other deviation from the grammar are errors. The parameter limit bounds the work that parsing, comparing, and hashing untrusted input can cause: duplicate detection and order-independent equality grow with the square of the number of parameters. Serialization quotes and escapes values that are not tokens, and parsing a serialized value yields an equal value.

Matching between ranges and types ignores parameters. Equality compares names and parameters, ignoring parameter order.

`MediaRangeSet` membership absorbs ranges covered by a wildcard and becomes universal when `*/*` is added. Its `contains` accepts only a concrete `MediaType`, so checking an untrusted type against an allowlist cannot be satisfied by a wildcard; `matches` accepts a range and answers whether any member is compatible, so a `*/*` query matches any non-empty set and a `type/*` query matches when any member has that type. A set stores its members without parameters, none covered by another, and enumerates them by reference in insertion order; a range that absorbs members takes its own place in that order. Sets have no value equality, as their insertion order is not significant but is kept for enumeration. `MediaTypeMap` accepts only concrete media types as keys. An exact query finds its own entry; `type/*` finds the first registration of that type and `*/*` the first registration overall. Registering a type again replaces its key and value in place, keeping its position for wildcard queries.

The crate must not parse HTTP fields or order ranges by quality, and must not depend on an HTTP or web framework crate.

## Rationale

The user chose separate types so that wildcard mistakes are type errors rather than runtime checks, and agreed to port `protocol-media`'s set and map semantics. Where the Ruby gem was more permissive than the RFCs, this crate follows the RFCs: parameter names are case-insensitive (RFC 9110), names may not repeat (RFC 6838), and line folding inside quoted values is the HTTP layer's concern. Keeping `Accept` parsing and quality ordering in the HTTP layer follows the gem's content negotiation design, and the dependency rule follows the `base-web-*` family rule in `base-project`'s Package Organization guide.

## Scope

The `base-web-media-type` crate's public API: `MediaType`, `MediaRange`, `MediaRangeSet`, `MediaTypeMap` and its iterator, `MediaTypeError`, and their parsing, serialization, comparison, and matching behavior.

## Validation

The integration tests under `tests/` exercise the grammar's accepted and rejected forms, wildcard rules, case normalization, quoting and round trips, equality and hashing, the parameter limit and its timing on long inputs, set absorption and universality, and map ordering and replacement. Inspect `cargo tree -p base-web-media-type -e normal`, which must list no dependencies.
