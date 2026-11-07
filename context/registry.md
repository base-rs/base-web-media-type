# Registry

This guide explains how to look up registered media types with `base-web-media-type-registry`, and how its data is maintained.

## Looking up types

The registry maps media type names, filename extensions, and paths to records. Each record has its `MediaType`, the MIME content transfer encoding conventionally used for it, and its known extensions:

```rust
use base_web_media_type::MediaType;
use base_web_media_type_registry as registry;

let html = registry::for_path("site/index.html").expect("a registered extension");
assert_eq!(html.media_type(), &MediaType::TEXT_HTML);

let json = registry::get("application/json").expect("a registered type");
assert_eq!(json.extensions(), ["json", "map"]);
```

Lookups are case-insensitive and use static tables, without allocating. `for_extension` accepts an extension with or without one leading dot. `for_path` uses the final extension only, so look up compound extensions such as `warc.gz` with `for_extension`. When several types list an extension, the one ranked most common is returned.

The registry describes conventions, not the content of a file: never trust a type derived from an untrusted filename for security decisions, such as whether content may be rendered inline.

## Data source

The table in `registry/src/records.rs` is generated from a pinned release of the [mime-types-data](https://github.com/mime-types/mime-types-data) project, which combines the IANA registry with types and extensions in common use. `base_web_media_type_registry::SOURCE_VERSION` names the release. The data is MIT licensed; its notice is in `registry/data-license.md`.

The generator, the `registry:generate` task in the private `bake/` package:

- trims and lowercases names, and skips names that are not valid concrete media types, reporting them;
- lowercases and deduplicates extensions, and adds `map` to `application/json`, as the Ruby `protocol-media-registry` does;
- chooses each extension's record by `mime-types-data`'s sort priority, as the Ruby generator does.

## Updating the data

Download `data/mime-types.json` from a new release and regenerate the table:

```sh
curl -fLO https://raw.githubusercontent.com/mime-types/mime-types-data/vVERSION/data/mime-types.json
cargo bake registry:generate mime-types.json VERSION
```

Review the changes to `registry/src/records.rs`, update the expected `SOURCE_VERSION` in the registry's tests, and note the update in `releases.md`. Both crates are released together.
