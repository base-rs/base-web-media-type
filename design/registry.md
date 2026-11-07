# Media Type Registry

## Requirement

`base-web-media-type-registry` must provide registered media types with their conventional transfer encodings and filename extensions, generated from a pinned release of the `mime-types-data` project. The generated table is committed, records the release it came from, and is regenerated only through the Rust `registry:generate` task in the private `bake/` package; normal builds must not download data or need Ruby, Python, or other tools.

Lookups by media type name, parsed media type, extension, and path must be case-insensitive and must not allocate. Names that are not valid concrete media types after trimming are omitted from the table and reported by the generator. Each extension resolves to the record `mime-types-data` ranks as most common, using the same priority rules as the Ruby `protocol-media-registry` generator, including its additional `map` extension for `application/json`.

The registry package must include the `mime-types-data` copyright and MIT license notice. It may depend only on `base-web-media-type`, and is released together with it at the same version.

## Rationale

The user chose `mime-types-data` as the source, as in the Ruby registry, because the IANA registry lists no filename extensions, and agreed to keep the registry as a crate in the same workspace as `base-web-media-type`. The organization's tooling is Rust; a committed table keeps builds offline and reviewable. The source data is MIT licensed, which requires its notice in copies.

## Scope

The `base-web-media-type-registry` crate, its generated `registry/src/records.rs`, and the `registry:generate` task.

## Validation

The registry's tests check case-insensitive lookups, extension priority for common and conflicting extensions, path handling, and that records are sorted and unique. The generator's tests in `bake/src/registry/tests.rs` check normalization, skipped names, priority rules, and rendering. Comparing the generated extension choices with the Ruby gem's table for the extensions both contain should show no differences; at the time of writing, none differed. Inspect `cargo tree -p base-web-media-type-registry -e normal` and the packaged file list from `cargo package --list -p base-web-media-type-registry`.
