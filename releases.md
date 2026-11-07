# Releases

## Unreleased

- Add `MediaType` and `MediaRange`, parsed and serialized following RFC 9110, with case-insensitive names, parameter access, wildcard rules, at most 64 parameters, and compile-time-checked constants.
- Add `MediaRangeSet`, which checks concrete types against an allowlist with `contains` and compatibility with ranges with `matches`, and `MediaTypeMap`, which selects a value for the first acceptable range in content negotiation.
- Add `base-web-media-type-registry`, with registered media types, transfer encodings, and filename extensions generated from mime-types-data 3.2026.1103, and `MediaTypeRecord` lookups by name, type, extension, and path: `by_name`, `by_media_type`, `by_extension`, and `by_path`.
