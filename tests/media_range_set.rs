// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use base_web_media_type::{MediaRange, MediaRangeSet, MediaType};

fn range(text: &str) -> MediaRange {
    MediaRange::parse(text).unwrap()
}

fn set(texts: &[&str]) -> MediaRangeSet {
    texts.iter().map(|text| range(text)).collect()
}

fn names(set: &MediaRangeSet) -> Vec<String> {
    set.iter().map(|range| range.to_string()).collect()
}

#[test]
fn contains_concrete_member_types() {
    let set = set(&["image/*", "application/pdf"]);
    for member in [
        MediaType::IMAGE_PNG,
        MediaType::IMAGE_JPEG,
        MediaType::APPLICATION_PDF,
        MediaType::parse("application/pdf; version=2").unwrap(),
    ] {
        assert!(set.contains(&member), "{member}");
    }
    for outside in [MediaType::TEXT_PLAIN, MediaType::APPLICATION_JSON] {
        assert!(!set.contains(&outside), "{outside}");
    }
    assert!(!set.is_universal() && !set.is_empty());
}

#[test]
fn matches_compatible_ranges() {
    let set = set(&["image/*", "application/pdf"]);
    for compatible in ["image/png", "image/*", "application/*", "*/*"] {
        assert!(set.matches(range(compatible)), "{compatible}");
    }
    for incompatible in ["text/plain", "text/*", "application/json"] {
        assert!(!set.matches(range(incompatible)), "{incompatible}");
    }
    // A wildcard query is not membership: it asks whether any member is compatible.
    let pngs = self::set(&["image/png"]);
    assert!(pngs.matches(range("image/*")) && pngs.matches(MediaRange::ANY));
}

#[test]
fn a_global_wildcard_makes_the_set_universal() {
    let mut set: MediaRangeSet = [MediaType::IMAGE_PNG].into_iter().collect();
    assert!(!set.is_universal());
    set.insert(MediaRange::ANY);
    set.insert(MediaType::TEXT_PLAIN);
    assert!(set.is_universal());
    assert!(set.contains(&MediaType::APPLICATION_JSON) && set.matches(range("text/*")));
    assert_eq!((names(&set), set.len()), (vec!["*/*".to_owned()], 1));
    assert!(!set.is_empty());
}

#[test]
fn enumerates_canonical_members_without_redundant_entries() {
    let set = set(&["application/pdf", "image/*; profile=example"]);
    assert_eq!(names(&set), ["application/pdf", "image/*"]);
    assert_eq!((set.len(), set.iter().len()), (2, 2));
    assert_eq!(
        (&set).into_iter().collect::<Vec<_>>(),
        [
            &MediaType::APPLICATION_PDF.into(),
            &MediaRange::from_static("image/*")
        ]
    );
    assert_eq!(
        set.clone().into_iter().collect::<Vec<_>>(),
        [
            MediaType::APPLICATION_PDF.into(),
            MediaRange::from_static("image/*")
        ]
    );
    // A wildcard replaces the members it covers and takes its own place in insertion order.
    assert_eq!(
        names(&self::set(&[
            "image/png",
            "application/pdf",
            "image/jpeg",
            "image/*"
        ])),
        ["application/pdf", "image/*"]
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
    let mut set = MediaRangeSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
    assert!(!set.matches(MediaRange::ANY) && !set.contains(&MediaType::TEXT_PLAIN));
    set.extend([MediaType::APPLICATION_JSON]);
    set.extend([range("text/*")]);
    assert!(set.contains(&MediaType::TEXT_PLAIN) && set.contains(&MediaType::APPLICATION_JSON));
    assert_eq!(names(&set.clone()), ["application/json", "text/*"]);
    assert_eq!(
        format!("{set:?}"),
        "{MediaRange(application/json), MediaRange(text/*)}"
    );
}

#[test]
fn wildcards_absorb_members_and_keep_the_rest_in_order() {
    let mut set = set(&[
        "text/plain",
        "image/png",
        "text/html",
        "image/gif",
        "application/json",
    ]);
    set.insert(range("text/*"));
    assert_eq!(
        names(&set),
        ["image/png", "image/gif", "application/json", "text/*"]
    );
    assert_eq!((set.len(), set.iter().len()), (4, 4));
    assert!(set.matches(range("text/*")) && set.contains(&MediaType::TEXT_CSS));
    set.insert(range("text/csv"));
    set.insert(range("image/*"));
    // Absorbed members now outnumber the members, so the set compacts and its index follows.
    assert_eq!(names(&set), ["application/json", "text/*", "image/*"]);
    set.insert(range("application/pdf"));
    set.insert(range("application/json"));
    assert_eq!(
        names(&set),
        ["application/json", "text/*", "image/*", "application/pdf"]
    );
    set.insert(range("application/*"));
    assert_eq!(names(&set), ["text/*", "image/*", "application/*"]);
    assert_eq!(
        set.into_iter().collect::<Vec<_>>(),
        [range("text/*"), range("image/*"), range("application/*")]
    );
}

#[test]
fn iterators_report_their_remaining_length() {
    let set = set(&["text/plain", "text/html", "image/*"]);
    let mut iterator = set.iter();
    assert_eq!(iterator.size_hint(), (3, Some(3)));
    assert_eq!(iterator.next(), Some(&range("text/plain")));
    assert_eq!(iterator.len(), 2);
    assert!(format!("{:?}", iterator.clone()).starts_with("MediaRangeSetIter"));
    assert_eq!(iterator.by_ref().count(), 2);
    assert_eq!(iterator.next(), None);
}

#[test]
fn large_sets_of_distinct_ranges_build_quickly() {
    // Each insertion is a constant-time index lookup, so a set built from a large untrusted `Accept` field costs time proportional to its size.
    let ranges: Vec<MediaRange> = (0..50_000)
        .map(|index| range(&format!("type{index}/subtype")))
        .collect();
    let set: MediaRangeSet = ranges.iter().collect();
    assert_eq!(set.len(), 50_000);
    assert!(set.contains(&MediaType::parse("type49999/subtype").unwrap()));
    assert!(set.matches(range("type7/*")) && !set.matches(range("type50000/*")));
}
