# `base-web-media-type`

Media types and media ranges for Rust, with sets and maps that match by range.

A `MediaType` names one kind of content, such as `text/html; charset=utf-8`; a `MediaRange` can also use wildcards, such as `text/*`. Both follow the grammar of RFC 9110. `MediaRangeSet` answers whether a type is acceptable, and `MediaTypeMap` finds a value, such as a renderer, for the first acceptable range in content negotiation. The `base-web-media-type-registry` crate in this workspace maps names and filename extensions to registered types.

## Usage

```rust
use base_web_media_type::{MediaRange, MediaType};

let html = MediaType::parse("text/html; charset=utf-8").unwrap();
assert!(MediaRange::new("text", "*").unwrap().matches(&html));
```

See the guides:

- [Getting Started](context/getting-started.md): parsing, building, comparing, and matching types, sets, and maps.
- [Content Negotiation](context/content-negotiation.md): selecting a representation from an HTTP `Accept` field's ranges.
- [Registry](context/registry.md): looking up types by name, extension, or path, and updating the registry's data.

## Releases

See [releases.md](releases.md).

## See Also

- [Protocol::Media](https://github.com/socketry/protocol-media) and [Protocol::Media::Registry](https://github.com/socketry/protocol-media-registry), the Ruby gems this crate ports.
- [mime-types-data](https://github.com/mime-types/mime-types-data), the source of the registry's data.

## Contributing

Please open an issue or pull request on [GitHub](https://github.com/base-rs/base-web-media-type).

Install the [Bake launcher](https://github.com/base-rs/bake) and run `cargo bake test`. Use `cargo bake test:coverage --all-targets true` for the shared source-region coverage gate.

### Agent Context

Run `cargo bake agent:context:install` to install shared context and skills. Use `cargo bake agent:context:list` and `cargo bake agent:context:show` to discover relevant guides, follow repository guidance in `.agents/`, and apply skills under `.agents/skills/`.

Before planning or changing behavior, discover and read applicable `design/*.md`. Use `base-design-review` to assess plans and changes, and `base-design-author` when user requirements are introduced, changed, or removed. Review design compliance before opening or updating a pull request.
