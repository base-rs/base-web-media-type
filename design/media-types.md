# Media Types and Ranges

## Requirement

Concrete media types and media ranges must be distinct types. A `MediaType` never contains a wildcard; a `MediaRange` may use `*` as a whole type or subtype, and a wildcard type requires a wildcard subtype. A `*` inside a name is an ordinary token character.

Parsing must follow the `media-type` grammar of RFC 9110 (sections 5.6 and 8.3.1) without accepting surrounding whitespace: tokens for names, token or quoted-string parameter values, optional whitespace around semicolons, and empty parameters. Type, subtype, and parameter names are case-insensitive and normalized to lowercase; parameter values keep their case and order. A repeated parameter name, a quoted value containing a control character other than horizontal tab, and any other deviation from the grammar are errors. Serialization quotes and escapes values that are not tokens, and parsing a serialized value yields an equal value.

Matching between ranges and types ignores parameters. Equality compares names and parameters, ignoring parameter order.

`MediaRangeSet` membership absorbs ranges covered by a wildcard, becomes universal when `*/*` is added, and answers a `*/*` query with whether it is non-empty and a `type/*` query with whether any member has that type. `MediaTypeMap` accepts only concrete media types as keys. An exact query finds its own entry; `type/*` finds the first registration of that type and `*/*` the first registration overall. Registering a type again replaces its key and value in place, keeping its position for wildcard queries.

The crate must not parse HTTP fields or order ranges by quality, and must not depend on an HTTP or web framework crate.

## Rationale

The user chose separate types so that wildcard mistakes are type errors rather than runtime checks, and agreed to port `protocol-media`'s set and map semantics. Where the Ruby gem was more permissive than the RFCs, this crate follows the RFCs: parameter names are case-insensitive (RFC 9110), names may not repeat (RFC 6838), and line folding inside quoted values is the HTTP layer's concern. Keeping `Accept` parsing and quality ordering in the HTTP layer follows the gem's content negotiation design, and the dependency rule follows the `base-web-*` family rule in `base-project`'s Package Organization guide.

## Scope

The `base-web-media-type` crate's public API: `MediaType`, `MediaRange`, `MediaRangeSet`, `MediaTypeMap`, `Error`, and their parsing, serialization, comparison, and matching behavior.

## Validation

The integration tests under `tests/` exercise the grammar's accepted and rejected forms, wildcard rules, case normalization, quoting and round trips, equality and hashing, set absorption and universality, and map ordering and replacement. Inspect `cargo tree -p base-web-media-type -e normal`, which must list no dependencies.
