// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use base_web_media_type::{MediaRange, MediaType, MediaTypeMap};

fn range(text: &str) -> MediaRange {
    MediaRange::parse(text).unwrap()
}

fn map() -> MediaTypeMap<&'static str> {
    [
        (MediaType::APPLICATION_JSON, "json"),
        (MediaType::TEXT_HTML, "html"),
        (MediaType::TEXT_PLAIN, "plain"),
    ]
    .into_iter()
    .collect()
}

#[test]
fn looks_up_exact_types_and_wildcards_by_registration_order() {
    let map = map();
    assert_eq!(map.get(MediaType::APPLICATION_JSON), Some(&"json"));
    assert_eq!(map.get(range("text/plain")), Some(&"plain"));
    assert_eq!(
        map.get(range("text/*")),
        Some(&"html"),
        "the first text registration"
    );
    assert_eq!(
        map.get(MediaRange::ANY),
        Some(&"json"),
        "the first registration"
    );
    assert_eq!(
        map.get(range("application/json; version=2")),
        Some(&"json"),
        "parameters are ignored"
    );
    assert_eq!(map.get(MediaType::IMAGE_PNG), None);
    assert_eq!(map.get(range("image/*")), None);
    assert_eq!(map.get(range("text/csv")), None);
    assert_eq!(
        map.get_key_value(range("text/*")),
        Some((&MediaType::TEXT_HTML, &"html"))
    );
    assert_eq!((map.len(), map.is_empty()), (3, false));
    assert_eq!(
        map.iter()
            .map(|(media_type, value)| (media_type.essence().to_owned(), *value))
            .collect::<Vec<_>>(),
        [
            ("application/json".to_owned(), "json"),
            ("text/html".to_owned(), "html"),
            ("text/plain".to_owned(), "plain")
        ]
    );
}

#[test]
fn replacing_a_registration_keeps_its_position() {
    let mut map = MediaTypeMap::new();
    assert_eq!(map.insert(MediaType::APPLICATION_JSON, "json"), None);
    assert_eq!(map.insert(MediaType::APPLICATION_PDF, "pdf"), None);
    let versioned = MediaType::parse("application/json; version=2").unwrap();
    assert_eq!(map.insert(versioned.clone(), "versioned"), Some("json"));
    assert_eq!(map.get(MediaType::APPLICATION_JSON), Some(&"versioned"));
    assert_eq!(map.get(range("application/*")), Some(&"versioned"));
    assert_eq!(map.get(MediaRange::ANY), Some(&"versioned"));
    assert_eq!(
        map.get_key_value(MediaType::APPLICATION_JSON),
        Some((&versioned, &"versioned")),
        "the key is replaced too"
    );
    assert_eq!(map.len(), 2);
}

#[test]
fn selects_the_first_acceptable_range() {
    let map = map();
    let accepted = [range("image/*"), range("text/plain; format=flowed")];
    let (value, selected) = map.select(&accepted).unwrap();
    assert_eq!((*value, selected), ("plain", &accepted[1]));
    let (value, selected) = map
        .select(vec![range("image/*"), MediaType::APPLICATION_JSON.into()])
        .unwrap();
    assert_eq!((*value, selected.essence()), ("json", "application/json"));
    assert!(map.select([range("image/*")]).is_none());
    assert!(
        MediaTypeMap::<()>::default()
            .select([MediaRange::ANY])
            .is_none()
    );
}

#[test]
fn an_empty_map_matches_nothing() {
    let mut map = MediaTypeMap::default();
    assert!(map.is_empty());
    assert_eq!(map.get(MediaRange::ANY), None::<&u8>);
    map.extend([(MediaType::IMAGE_PNG, 1)]);
    assert_eq!(map.get(MediaRange::ANY), Some(&1));
    assert_eq!(format!("{:?}", map.clone()), "{MediaType(image/png): 1}");
}

#[test]
fn borrowed_maps_iterate_in_registration_order() {
    let map = map();
    let mut iterator = (&map).into_iter();
    assert_eq!(iterator.len(), 3);
    assert_eq!(
        iterator.next(),
        Some((&MediaType::APPLICATION_JSON, &"json"))
    );
    assert_eq!(iterator.size_hint(), (2, Some(2)));
    let values: Vec<_> = iterator.clone().map(|(_, value)| *value).collect();
    assert_eq!(values, ["html", "plain"]);
    assert!(format!("{iterator:?}").starts_with("MediaTypeMapIter"));
    let mut count = 0;
    for (media_type, _) in &map {
        assert!(media_type.as_range().is_concrete());
        count += 1;
    }
    assert_eq!(count, 3);
}
