# Releases

## Unreleased

- Add `MediaType` and `MediaRange`, parsed and serialized following RFC 9110, with case-insensitive names, parameter access, wildcard rules, and compile-time-checked constants.
- Add `MediaRangeSet`, which answers whether a type or range is acceptable, and `MediaTypeMap`, which selects a value for the first acceptable range in content negotiation.
- Add `base-web-media-type-registry`, with registered media types, transfer encodings, and filename extensions generated from mime-types-data 3.2026.1103, and lookups by name, type, extension, and path.
