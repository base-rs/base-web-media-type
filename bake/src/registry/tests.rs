// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use super::*;

const SOURCE: &str = r#"[
    {"content-type": "text/plain", "encoding": "quoted-printable", "extensions": ["txt", "TXT", "text"], "sort-priority": 2},
    {"content-type": "application/json", "encoding": "8bit", "extensions": ["json", "map", ".htaccess", "."]},
    {"content-type": "Application/Vnd.Example ", "encoding": "7bit", "extensions": ["txt"], "sort-priority": 1},
    {"content-type": "application/x-compress", "encoding": "binary", "extensions": ["z"], "sort-priority": 174, "preferred-extension": "z"},
    {"content-type": "application/x-compressed", "encoding": "base64", "extensions": ["z"], "sort-priority": 46},
    {"content-type": "application/application/broken", "encoding": "base64"}
]"#;

#[test]
fn records_are_normalized_and_sorted() {
    let (records, skipped) = records(SOURCE).unwrap();
    let names: Vec<_> = records.iter().map(Record::name).collect();
    assert_eq!(
        names,
        [
            "application/json",
            "application/vnd.example",
            "application/x-compress",
            "application/x-compressed",
            "text/plain"
        ]
    );
    assert_eq!(skipped, ["\"application/application/broken\""]);
    assert_eq!(
        records[0].extensions,
        ["json", "map", "htaccess"],
        "leading dots are removed, empty extensions skipped, and an existing additional extension is not repeated"
    );
    assert_eq!(
        records[4].extensions,
        ["txt", "text"],
        "extensions are lowercased and deduplicated"
    );
    assert_eq!(
        (records[4].encoding, records[0].priority),
        ("QuotedPrintable", 0xff)
    );
}

#[test]
fn each_extension_prefers_the_lowest_priority() {
    let (records, _) = records(SOURCE).unwrap();
    let extensions = extensions(&records);
    // `txt` is listed by priority 1 and 2; `z` by 46 and by a preferred-extension record at 174, promoted to 167.
    assert_eq!(
        extensions.into_iter().collect::<Vec<_>>(),
        [
            ("htaccess", 0),
            ("json", 0),
            ("map", 0),
            ("text", 4),
            ("txt", 1),
            ("z", 3)
        ]
    );
}

#[test]
fn renders_rust_tables() {
    let (records, _) = records(SOURCE).unwrap();
    let output = render(&records, "3.2026.1103").unwrap();
    assert!(output.contains("from mime-types-data 3.2026.1103; do not edit."));
    assert!(output.contains("pub(crate) const SOURCE_VERSION: &str = \"3.2026.1103\";"));
    assert!(output.contains("pub(crate) static RECORDS: [Record; 5] = [\n    Record::new(\"application/json\", EightBit, &[\"json\", \"map\", \"htaccess\"]),"));
    assert!(output
        .contains("    Record::new(\"text/plain\", QuotedPrintable, &[\"txt\", \"text\"]),\n];"));
    assert!(output
        .contains("pub(crate) static EXTENSIONS: [(&str, u16); 6] = [\n    (\"htaccess\", 0),"));
    let many: Vec<_> = (0..=u32::from(u16::MAX))
        .map(|index| Record {
            top_level_type: "a".to_owned(),
            subtype: index.to_string(),
            encoding: "Base64",
            extensions: Vec::new(),
            priority: 0,
            preferred_extension: None,
        })
        .collect();
    assert!(render(&many, "1")
        .unwrap_err()
        .to_string()
        .contains("too many records"));
    for version in ["", "3..1", "3.x", "v3"] {
        assert!(render(&records, version).is_err(), "{version}");
    }
}

#[test]
fn invalid_sources_are_errors() {
    let cases = [
        ("not json", "invalid mime-types-data JSON"),
        (r#"[{"encoding": "8bit"}]"#, "no content-type"),
        (
            r#"[{"content-type": "a/b", "encoding": "utf-9"}]"#,
            "unknown encoding",
        ),
        (
            r#"[{"content-type": "a/b", "encoding": "8bit", "extensions": [1]}]"#,
            "not a string",
        ),
        (
            r#"[{"content-type": "a/b", "encoding": "8bit"}, {"content-type": "A/B", "encoding": "8bit"}]"#,
            "duplicate media type",
        ),
        (
            r#"[{"content-type": "a/b", "encoding": "8bit"}]"#,
            "missing media type application/json",
        ),
    ];
    for (source, message) in cases {
        let error = records(source).unwrap_err().to_string();
        assert!(error.contains(message), "{message}: {error}");
    }
}

#[test]
fn the_task_writes_the_generated_table() {
    let directory = std::env::temp_dir().join(format!("registry-generate-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("registry/src")).unwrap();
    std::fs::write(directory.join("source.json"), SOURCE).unwrap();
    let mut context = bake::Registry::discover().unwrap().context(&directory);
    let summary = context
        .call("registry:generate", &["source.json", "3.1"])
        .unwrap();
    assert_eq!(summary, "Generated 5 records and 6 extensions from mime-types-data 3.1; skipped invalid media types: \"application/application/broken\"");
    assert!(std::fs::read_to_string(directory.join(OUTPUT))
        .unwrap()
        .contains("Record::new(\"text/plain\""));
    std::fs::write(
        directory.join("clean.json"),
        r#"[{"content-type": "application/json", "encoding": "8bit"}]"#,
    )
    .unwrap();
    assert_eq!(
        context
            .call("registry:generate", &["clean.json", "3.2"])
            .unwrap(),
        "Generated 1 records and 1 extensions from mime-types-data 3.2"
    );
    assert!(context
        .call("registry:generate", &["missing.json", "3.1"])
        .unwrap_err()
        .to_string()
        .contains("missing.json"));
    assert!(context
        .call("registry:generate", &["source.json", "bad"])
        .is_err());
    std::fs::write(directory.join("invalid.json"), "not json").unwrap();
    assert!(context
        .call("registry:generate", &["invalid.json", "3.1"])
        .is_err());
    std::fs::remove_dir_all(directory.join("registry")).unwrap();
    assert!(
        context
            .call("registry:generate", &["source.json", "3.1"])
            .is_err(),
        "the output directory must exist"
    );
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn records_are_ordered_by_type_then_subtype() {
    // `x-foo/a` sorts before `x/a` as a string, but after it as a type and subtype pair, which is how lookups search.
    let source = r#"[{"content-type": "x-foo/a", "encoding": "8bit"}, {"content-type": "x/a", "encoding": "8bit"}, {"content-type": "application/json", "encoding": "8bit"}]"#;
    let (records, _) = records(source).unwrap();
    let names: Vec<_> = records.iter().map(Record::name).collect();
    assert_eq!(names, ["application/json", "x/a", "x-foo/a"]);
    assert_eq!(records[2].encoding, "EightBit");
}
