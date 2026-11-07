# `base-web-media-type-registry`

Registered media types for Rust, with their conventional transfer encodings and filename extensions.

```rust
use base_web_media_type::MediaType;
use base_web_media_type_registry as registry;

assert_eq!(registry::for_path("index.html").unwrap().media_type(), &MediaType::TEXT_HTML);
assert!(registry::get("application/json").unwrap().extensions().contains(&"json"));
```

Lookups by name, parsed type, extension, and path are case-insensitive and do not allocate. The data is generated from a pinned release of [mime-types-data](https://github.com/mime-types/mime-types-data), available as `SOURCE_VERSION`. See the [Registry guide](https://github.com/base-rs/base-web-media-type/blob/main/context/registry.md) for details and the data update procedure.

This crate is released together with [`base-web-media-type`](https://github.com/base-rs/base-web-media-type) at the same version. The source data's license notice is in [data-license.md](data-license.md).
