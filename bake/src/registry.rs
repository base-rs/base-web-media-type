// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! Generate the registry's static tables from the `mime-types-data` project's `data/mime-types.json`.

use bake::{Context, Error, Result};
use base_web_media_type::MediaType;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

/// The generated table, relative to the project root.
const OUTPUT: &str = "registry/src/records.rs";

/// Extensions added to the source data, as in the Ruby registry: source maps are JSON.
const ADDITIONAL_EXTENSIONS: [(&str, &[&str]); 1] = [("application/json", &["map"])];

/// Generate `registry/src/records.rs` from a `mime-types-data` release.
///
/// Download `data/mime-types.json` from the release, for example `https://raw.githubusercontent.com/mime-types/mime-types-data/v{version}/data/mime-types.json`, then pass its path and the release version.
#[bake::task]
pub fn generate(
    context: &mut Context,
    #[bake(help = "Path to the release's data/mime-types.json, relative to the project root.")]
    source: PathBuf,
    #[bake(help = "The mime-types-data release version, such as 3.2026.1103.")] version: String,
) -> Result<String> {
    let text = std::fs::read_to_string(context.root().join(&source))
        .map_err(|error| Error::new(format!("{}: {error}", source.display())))?;
    let (records, skipped) = records(&text)?;
    let output = render(&records, &version)?;
    std::fs::write(context.root().join(OUTPUT), output)?;
    let mut summary = format!(
        "Generated {} records and {} extensions from mime-types-data {version}",
        records.len(),
        extensions(&records).len()
    );
    if !skipped.is_empty() {
        write!(
            summary,
            "; skipped invalid media types: {}",
            skipped.join(", ")
        )
        .expect("writing to a string cannot fail");
    }
    Ok(summary)
}

/// One media type from the source data.
#[derive(Debug, PartialEq)]
pub(crate) struct Record {
    pub top_level_type: String,
    pub subtype: String,
    pub encoding: &'static str,
    pub extensions: Vec<String>,
    pub priority: u64,
    pub preferred_extension: Option<String>,
}

impl Record {
    fn name(&self) -> String {
        format!("{}/{}", self.top_level_type, self.subtype)
    }
}

/// Parse the source records, sorted by type and subtype. Names are trimmed and lowercased; names that are not concrete media types are skipped and returned.
pub(crate) fn records(text: &str) -> Result<(Vec<Record>, Vec<String>)> {
    let source: Vec<Value> = serde_json::from_str(text)
        .map_err(|error| Error::new(format!("invalid mime-types-data JSON: {error}")))?;
    let mut records = BTreeMap::new();
    let mut skipped = Vec::new();
    for record in &source {
        let name = record["content-type"]
            .as_str()
            .ok_or_else(|| Error::new("a record has no content-type"))?;
        let Ok(media_type) = MediaType::parse(name.trim()) else {
            skipped.push(format!("{name:?}"));
            continue;
        };
        let encoding = match record["encoding"].as_str() {
            Some("7bit") => "SevenBit",
            Some("8bit") => "EightBit",
            Some("binary") => "Binary",
            Some("base64") => "Base64",
            Some("quoted-printable") => "QuotedPrintable",
            other => return Err(Error::new(format!("{name}: unknown encoding {other:?}"))),
        };
        let mut extensions: Vec<String> = Vec::new();
        for extension in record["extensions"].as_array().into_iter().flatten() {
            let extension = extension
                .as_str()
                .ok_or_else(|| Error::new(format!("{name}: an extension is not a string")))?
                .trim_start_matches('.')
                .to_ascii_lowercase();
            if !extension.is_empty() && !extensions.contains(&extension) {
                extensions.push(extension);
            }
        }
        let record = Record {
            top_level_type: media_type.top_level_type().to_owned(),
            subtype: media_type.subtype().to_owned(),
            encoding,
            extensions,
            priority: record["sort-priority"].as_u64().unwrap_or(0xff),
            preferred_extension: record["preferred-extension"]
                .as_str()
                .map(str::to_ascii_lowercase),
        };
        if records
            .insert(
                (record.top_level_type.clone(), record.subtype.clone()),
                record,
            )
            .is_some()
        {
            return Err(Error::new(format!("duplicate media type {name}")));
        }
    }
    for (name, additions) in ADDITIONAL_EXTENSIONS {
        let (top_level_type, subtype) = name
            .split_once('/')
            .expect("additional extensions name media types");
        let record = records
            .get_mut(&(top_level_type.to_owned(), subtype.to_owned()))
            .ok_or_else(|| {
                Error::new(format!(
                    "cannot add extensions to missing media type {name}"
                ))
            })?;
        for addition in additions {
            if !record
                .extensions
                .iter()
                .any(|extension| extension == addition)
            {
                record.extensions.push((*addition).to_owned());
            }
        }
    }
    Ok((records.into_values().collect(), skipped))
}

/// Each extension with the index of its preferred record, sorted by extension.
///
/// As in `mime-types-data`'s own ordering, the record with the lowest sort priority wins, ties going to the lower name. A record whose preferred extension this is, and whose priority marks it as not preferred (bit 3), is promoted to the best priority in its class.
pub(crate) fn extensions(records: &[Record]) -> BTreeMap<&str, usize> {
    let mut preferred: BTreeMap<&str, (u64, usize)> = BTreeMap::new();
    for (index, record) in records.iter().enumerate() {
        let preferred_extension = record
            .preferred_extension
            .as_deref()
            .or(record.extensions.first().map(String::as_str));
        for extension in &record.extensions {
            let mut priority = record.priority;
            if preferred_extension == Some(extension.as_str()) && priority & 0b1000 != 0 {
                priority = (priority & 0b1111_0111) | 0b0111;
            }
            // Records are sorted by name, so the first of equal priority has the lower name.
            let current = preferred.entry(extension).or_insert((priority, index));
            if priority < current.0 {
                *current = (priority, index);
            }
        }
    }
    preferred
        .into_iter()
        .map(|(extension, (_, index))| (extension, index))
        .collect()
}

/// The Rust source of the generated tables.
pub(crate) fn render(records: &[Record], version: &str) -> Result<String> {
    if !version
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(Error::new(format!(
            "invalid mime-types-data version {version:?}"
        )));
    }
    if u16::try_from(records.len()).is_err() {
        return Err(Error::new("too many records for 16-bit extension indexes"));
    }
    let extensions = extensions(records);
    let mut output = String::new();
    writeln!(output, "// Generated by `cargo bake registry:generate` from mime-types-data {version}; do not edit.\n").expect("writing to a string cannot fail");
    writeln!(
        output,
        "use crate::ContentTransferEncoding::*;\nuse crate::MediaTypeRecord;\n"
    )
    .expect("writing to a string cannot fail");
    writeln!(
        output,
        "pub(crate) const SOURCE_VERSION: &str = {version:?};\n"
    )
    .expect("writing to a string cannot fail");
    writeln!(
        output,
        "pub(crate) static RECORDS: [MediaTypeRecord; {}] = [",
        records.len()
    )
    .expect("writing to a string cannot fail");
    for record in records {
        writeln!(
            output,
            "    MediaTypeRecord::new({:?}, {}, &{:?}),",
            record.name(),
            record.encoding,
            record.extensions
        )
        .expect("writing to a string cannot fail");
    }
    writeln!(
        output,
        "];\n\npub(crate) static EXTENSIONS: [(&str, u16); {}] = [",
        extensions.len()
    )
    .expect("writing to a string cannot fail");
    for (extension, index) in extensions {
        writeln!(output, "    ({extension:?}, {index}),").expect("writing to a string cannot fail");
    }
    output.push_str("];\n");
    Ok(output)
}

#[cfg(test)]
mod tests;
