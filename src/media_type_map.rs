// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::{MediaRange, MediaType};
use std::collections::HashMap;
use std::fmt;

/// A map from concrete media types to values, looked up by media range, such as the representations an application can produce for content negotiation.
///
/// Lookup ignores parameters. An exact type finds its own entry. A wildcard query finds the first registration that matches it: `type/*` the first entry of that type, and `*/*` the first entry overall, so registration order sets the defaults. Registering a type again replaces its key and value but keeps its original position.
///
/// ```
/// use base_web_media_type::{MediaRange, MediaType, MediaTypeMap};
///
/// let mut renderers = MediaTypeMap::new();
/// renderers.insert(MediaType::APPLICATION_JSON, "json");
/// renderers.insert(MediaType::TEXT_PLAIN, "text");
///
/// // The client's ranges, in preference order, as parsed from an `Accept` field.
/// let accepted = [MediaRange::parse("image/*")?, MediaRange::parse("text/*")?];
/// let (renderer, range) = renderers.select(&accepted).unwrap();
/// assert_eq!((*renderer, range.subtype()), ("text", "*"));
/// # Ok::<(), base_web_media_type::Error>(())
/// ```
#[derive(Clone)]
pub struct MediaTypeMap<T> {
    entries: Vec<(MediaType, T)>,
    /// Entry indexes by top-level type: the first entry of the type, and each subtype's entry.
    types: HashMap<String, TypeIndex>,
}

#[derive(Clone)]
struct TypeIndex {
    first: usize,
    subtypes: HashMap<String, usize>,
}

impl<T> MediaTypeMap<T> {
    /// An empty map.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            types: HashMap::new(),
        }
    }

    /// Associate `media_type` with `value`, returning the value it replaces. A replaced entry keeps its position.
    pub fn insert(&mut self, media_type: MediaType, value: T) -> Option<T> {
        let range = media_type.as_range();
        let index = self.entries.len();
        let type_index = self
            .types
            .entry(range.top_level_type().to_owned())
            .or_insert_with(|| TypeIndex {
                first: index,
                subtypes: HashMap::new(),
            });
        match type_index.subtypes.get(range.subtype()) {
            Some(&existing) => {
                let entry = &mut self.entries[existing];
                entry.0 = media_type;
                Some(std::mem::replace(&mut entry.1, value))
            }
            None => {
                type_index
                    .subtypes
                    .insert(range.subtype().to_owned(), index);
                self.entries.push((media_type, value));
                None
            }
        }
    }

    /// The value for the entry compatible with `range`, as described for the map.
    pub fn get(&self, range: impl AsRef<MediaRange>) -> Option<&T> {
        self.entry(range.as_ref()).map(|(_, value)| value)
    }

    /// The registered media type and value compatible with `range`.
    pub fn get_key_value(&self, range: impl AsRef<MediaRange>) -> Option<(&MediaType, &T)> {
        self.entry(range.as_ref())
            .map(|(media_type, value)| (media_type, value))
    }

    /// The value for the first of `ranges`, in preference order, that has a compatible entry, together with that range.
    ///
    /// Returning the range lets the caller read its parameters, such as a requested version. Ordering ranges by preference, such as by an `Accept` field's quality values, is the caller's responsibility.
    pub fn select<R: AsRef<MediaRange>>(
        &self,
        ranges: impl IntoIterator<Item = R>,
    ) -> Option<(&T, R)> {
        ranges
            .into_iter()
            .find_map(|range| self.get(&range).map(|value| (value, range)))
    }

    /// The number of registered media types.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no media types are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Registered media types and values, in registration order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&MediaType, &T)> {
        self.entries
            .iter()
            .map(|(media_type, value)| (media_type, value))
    }

    fn entry(&self, range: &MediaRange) -> Option<&(MediaType, T)> {
        if range.is_wildcard_type() {
            return self.entries.first();
        }
        let type_index = self.types.get(range.top_level_type())?;
        let index = if range.is_wildcard_subtype() {
            type_index.first
        } else {
            *type_index.subtypes.get(range.subtype())?
        };
        Some(&self.entries[index])
    }
}

impl<T: fmt::Debug> fmt::Debug for MediaTypeMap<T> {
    /// Shows the entries in registration order, without the lookup index.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<T> Default for MediaTypeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<(MediaType, T)> for MediaTypeMap<T> {
    fn from_iter<I: IntoIterator<Item = (MediaType, T)>>(entries: I) -> Self {
        let mut map = Self::new();
        map.extend(entries);
        map
    }
}

impl<T> Extend<(MediaType, T)> for MediaTypeMap<T> {
    fn extend<I: IntoIterator<Item = (MediaType, T)>>(&mut self, entries: I) {
        for (media_type, value) in entries {
            self.insert(media_type, value);
        }
    }
}
