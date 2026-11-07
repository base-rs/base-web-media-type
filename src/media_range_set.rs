// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::MediaRange;
use std::borrow::Cow;

/// A set of media ranges, for asking whether a media type or range is acceptable, such as for allowed upload types.
///
/// Membership depends only on types and subtypes; parameters are ignored. Ranges covered by a wildcard are absorbed: adding `image/*` makes `image/png` redundant, and `*/*` makes the set universal.
///
/// ```
/// use base_web_media_type::{MediaRange, MediaRangeSet, MediaType};
///
/// let images: MediaRangeSet = [MediaRange::from_static("image", "*")].into_iter().collect();
/// assert!(images.contains(MediaType::IMAGE_PNG));
/// assert!(!images.contains(MediaType::TEXT_PLAIN));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    top_level_type: Cow<'static, str>,
    /// `None` when every subtype is a member; otherwise member subtypes in the order they were first added.
    subtypes: Option<Vec<Cow<'static, str>>>,
}

impl MediaRangeSet {
    /// A set containing `ranges`.
    pub fn new<R: AsRef<MediaRange>>(ranges: impl IntoIterator<Item = R>) -> Self {
        let mut set = Self::default();
        for range in ranges {
            set.insert(range);
        }
        set
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
        let subtype = (!range.is_wildcard_subtype()).then(|| range.subtype_name().clone());
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
                top_level_type: range.top_level_type_name().clone(),
                subtypes: subtype.map(|subtype| vec![subtype]),
            }),
        }
    }

    /// Whether a member is compatible with `range`. A query for `*/*` matches any non-empty set, and `type/*` matches when any member has that type.
    pub fn contains(&self, range: impl AsRef<MediaRange>) -> bool {
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
    pub fn iter(&self) -> impl Iterator<Item = MediaRange> + '_ {
        let (universal, types) = match &self.membership {
            Membership::Universal => (true, &[][..]),
            Membership::Types(types) => (false, types.as_slice()),
        };
        universal
            .then_some(MediaRange::ANY)
            .into_iter()
            .chain(types.iter().flat_map(|members| {
                let subtypes = match &members.subtypes {
                    None => vec![Cow::Borrowed("*")],
                    Some(subtypes) => subtypes.clone(),
                };
                subtypes
                    .into_iter()
                    .map(|subtype| MediaRange::from_names(members.top_level_type.clone(), subtype))
            }))
    }
}

impl<R: AsRef<MediaRange>> FromIterator<R> for MediaRangeSet {
    fn from_iter<I: IntoIterator<Item = R>>(ranges: I) -> Self {
        Self::new(ranges)
    }
}

impl<R: AsRef<MediaRange>> Extend<R> for MediaRangeSet {
    fn extend<I: IntoIterator<Item = R>>(&mut self, ranges: I) {
        for range in ranges {
            self.insert(range);
        }
    }
}
