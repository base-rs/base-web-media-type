// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::{MediaRange, MediaType};

/// A set of media ranges, such as the types an upload field accepts.
///
/// Membership depends only on types and subtypes; parameters are ignored. Ranges covered by a wildcard are absorbed: adding `image/*` makes `image/png` redundant, and `*/*` makes the set universal.
///
/// Use [`Self::contains`] to check a concrete media type, such as a request's `Content-Type`, against the set: parse it as a [`MediaType`], so a client cannot send a wildcard. Use [`Self::matches`] to ask whether any member is compatible with a range, such as in content negotiation.
///
/// ```
/// use base_web_media_type::{MediaRange, MediaRangeSet, MediaType};
///
/// let allowed: MediaRangeSet = [MediaRange::from_static("image/*")].into_iter().collect();
/// assert!(allowed.contains(&MediaType::IMAGE_PNG));
/// assert!(!allowed.contains(&MediaType::TEXT_PLAIN));
/// ```
#[derive(Clone, Debug, Default)]
pub struct MediaRangeSet {
    membership: Membership,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Membership {
    /// Every media range is a member.
    Universal,
    /// Member types in the order they were first added.
    Types(Vec<TypeMembers>),
}

impl Default for Membership {
    fn default() -> Self {
        Self::Types(Vec::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypeMembers {
    top_level_type: String,
    /// `None` when every subtype is a member; otherwise member subtypes in the order they were first added.
    subtypes: Option<Vec<String>>,
}

impl MediaRangeSet {
    /// An empty set. Build one from ranges with `collect` or `extend`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a range. Its parameters are ignored.
    pub fn insert(&mut self, range: impl AsRef<MediaRange>) {
        let range = range.as_ref();
        let Membership::Types(types) = &mut self.membership else {
            return;
        };
        if range.is_wildcard_type() {
            self.membership = Membership::Universal;
            return;
        }
        let subtype = (!range.is_wildcard_subtype()).then(|| range.subtype().to_owned());
        match types
            .iter_mut()
            .find(|members| members.top_level_type == range.top_level_type())
        {
            Some(members) => match (&mut members.subtypes, subtype) {
                (Some(_), None) => members.subtypes = None,
                (Some(subtypes), Some(subtype)) if !subtypes.contains(&subtype) => {
                    subtypes.push(subtype)
                }
                _ => {}
            },
            None => types.push(TypeMembers {
                top_level_type: range.top_level_type().to_owned(),
                subtypes: subtype.map(|subtype| vec![subtype]),
            }),
        }
    }

    /// Whether `media_type` is a member: a member range matches its type and subtype.
    pub fn contains(&self, media_type: &MediaType) -> bool {
        self.matches(media_type)
    }

    /// Whether any member is compatible with `range`. For a concrete type this is membership, as with [`Self::contains`]; a `type/*` query matches when any member has that type, and `*/*` matches any non-empty set.
    pub fn matches(&self, range: impl AsRef<MediaRange>) -> bool {
        let range = range.as_ref();
        let types = match &self.membership {
            Membership::Universal => return true,
            Membership::Types(types) => types,
        };
        if range.is_wildcard_type() {
            return !types.is_empty();
        }
        let Some(members) = types
            .iter()
            .find(|members| members.top_level_type == range.top_level_type())
        else {
            return false;
        };
        match &members.subtypes {
            None => true,
            Some(_) if range.is_wildcard_subtype() => true,
            Some(subtypes) => subtypes.iter().any(|subtype| subtype == range.subtype()),
        }
    }

    /// Whether every media range is a member, because `*/*` was added.
    pub fn is_universal(&self) -> bool {
        self.membership == Membership::Universal
    }

    /// Whether the set has no members.
    pub fn is_empty(&self) -> bool {
        self.membership == Membership::Types(Vec::new())
    }

    /// The number of canonical member ranges, as returned by [`Self::iter`].
    pub fn len(&self) -> usize {
        match &self.membership {
            Membership::Universal => 1,
            Membership::Types(types) => types
                .iter()
                .map(|members| members.subtypes.as_ref().map_or(1, Vec::len))
                .sum(),
        }
    }

    /// The canonical member ranges, without parameters or redundant entries: `*/*` alone for a universal set, otherwise `type/*` or each `type/subtype`, by type in the order each was first added.
    pub fn iter(&self) -> std::vec::IntoIter<MediaRange> {
        let ranges: Vec<MediaRange> = match &self.membership {
            Membership::Universal => vec![MediaRange::ANY],
            Membership::Types(types) => types
                .iter()
                .flat_map(|members| match &members.subtypes {
                    None => vec![MediaRange::from_parts(&members.top_level_type, "*")],
                    Some(subtypes) => subtypes
                        .iter()
                        .map(|subtype| MediaRange::from_parts(&members.top_level_type, subtype))
                        .collect(),
                })
                .collect(),
        };
        ranges.into_iter()
    }
}

impl IntoIterator for &MediaRangeSet {
    type Item = MediaRange;
    type IntoIter = std::vec::IntoIter<MediaRange>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R: AsRef<MediaRange>> FromIterator<R> for MediaRangeSet {
    fn from_iter<I: IntoIterator<Item = R>>(ranges: I) -> Self {
        let mut set = Self::new();
        set.extend(ranges);
        set
    }
}

impl<R: AsRef<MediaRange>> Extend<R> for MediaRangeSet {
    fn extend<I: IntoIterator<Item = R>>(&mut self, ranges: I) {
        for range in ranges {
            self.insert(range);
        }
    }
}
