// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use base_web_media_type::{MediaRange, MediaRangeSet, MediaType};

fn ranges(texts: &[&str]) -> Vec<MediaRange> {
    texts
        .iter()
        .map(|text| MediaRange::parse(text).unwrap())
        .collect()
}

fn set(texts: &[&str]) -> MediaRangeSet {
    ranges(texts).into_iter().collect()
}

fn names(set: &MediaRangeSet) -> Vec<String> {
    set.iter().map(|range| range.to_string()).collect()
}

#[test]
fn matches_compatible_types_and_ranges() {
    let set = set(&["image/*", "application/pdf"]);
    for member in [
        "image/png",
        "image/jpeg",
        "application/pdf",
        "application/pdf; version=2",
        "image/*",
        "application/*",
        "*/*",
    ] {
        assert!(set.contains(MediaRange::parse(member).unwrap()), "{member}");
    }
    assert!(set.contains(MediaType::IMAGE_PNG));
    for outside in ["text/plain", "text/*", "application/json"] {
        assert!(
            !set.contains(MediaRange::parse(outside).unwrap()),
            "{outside}"
        );
    }
    assert!(!set.is_universal() && !set.is_empty());
}

#[test]
fn a_global_wildcard_makes_the_set_universal() {
    let mut set = MediaRangeSet::new([MediaType::IMAGE_PNG]);
    assert!(
        set.contains(MediaRange::ANY),
        "a wildcard query matches any non-empty set"
    );
    assert!(!set.is_universal());
    set.insert(MediaRange::ANY);
    set.insert(MediaType::TEXT_PLAIN);
    assert!(set.is_universal());
    assert!(
        set.contains(MediaType::APPLICATION_JSON)
            && set.contains(MediaRange::parse("text/*").unwrap())
    );
    assert_eq!((names(&set), set.len()), (vec!["*/*".to_owned()], 1));
    assert!(!set.is_empty());
}

#[test]
fn enumerates_canonical_members_without_redundant_entries() {
    let set = set(&["application/pdf", "image/*; profile=example"]);
    assert_eq!(names(&set), ["application/pdf", "image/*"]);
    assert_eq!(set.len(), 2);
    assert_eq!(
        names(&self::set(&[
            "image/png",
            "application/pdf",
            "image/jpeg",
            "image/*"
        ])),
        ["image/*", "application/pdf"]
    );
    assert_eq!(
        names(&self::set(&["image/*", "application/pdf", "image/png"])),
        ["image/*", "application/pdf"]
    );
    assert_eq!(
        names(&self::set(&["image/*", "*/*", "text/plain"])),
        ["*/*"]
    );
    assert_eq!(
        names(&self::set(&["text/plain", "text/html", "text/plain"])),
        ["text/plain", "text/html"]
    );
}

#[test]
fn can_be_empty_and_extended() {
    let mut set = MediaRangeSet::default();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
    assert!(!set.contains(MediaRange::ANY) && !set.contains(MediaType::TEXT_PLAIN));
    set.extend([MediaType::APPLICATION_JSON]);
    set.extend(ranges(&["text/*"]));
    assert!(set.contains(MediaType::TEXT_PLAIN) && set.contains(MediaType::APPLICATION_JSON));
    assert_eq!(set.len(), 2);
    assert_eq!(set.clone(), set);
}
