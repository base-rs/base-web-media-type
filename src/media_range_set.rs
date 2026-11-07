// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::{MediaRange, MediaType};
use std::collections::HashMap;
use std::fmt;

/// A set of media ranges, such as the types an upload field accepts.
///
/// Membership depends only on types and subtypes: members are stored without parameters. Ranges covered by a wildcard are absorbed: adding `image/*` replaces `image/png`, and adding `*/*` replaces every member, making the set universal.
///
/// Use [`Self::contains`] to check a concrete media type, such as a request's `Content-Type`, against the set: parse it as a [`MediaType`], so a client cannot send a wildcard. Use [`Self::matches`] to ask whether any member is compatible with a range, such as in content negotiation.
///
/// Inserting and querying take constant expected time, however many members the set has, so a set can be built from untrusted ranges, such as those of an `Accept` field.
///
/// ```
/// use base_web_media_type::{MediaRange, MediaRangeSet, MediaType};
///
/// let allowed: MediaRangeSet = [MediaRange::from_static("image/*")].into_iter().collect();
/// assert!(allowed.contains(&MediaType::IMAGE_PNG));
/// assert!(!allowed.contains(&MediaType::TEXT_PLAIN));
/// ```
#[derive(Clone, Default)]
pub struct MediaRangeSet {
    /// Members without parameters, in the order they were added. `None` marks a member absorbed by a later wildcard.
    slots: Vec<Option<MediaRange>>,
    /// The number of members, the slots that are `Some`.
    len: usize,
    /// Whether `*/*` is a member, and so the only one.
    universal: bool,
    /// Members by top-level type, so that inserting and querying do not scan the members.
    types: HashMap<String, TypeMembers>,
}

#[derive(Clone, Default)]
struct TypeMembers {
    /// Whether `type/*` is a member. Its subtypes are then absorbed, so `subtypes` is empty.
    wildcard: bool,
    /// The slot of each member subtype.
    subtypes: HashMap<String, usize>,
}

impl MediaRangeSet {
    /// An empty set. Build one from ranges with `collect` or `extend`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a range, without its parameters, unless a member already covers it. Members it covers are removed.
    pub fn insert(&mut self, range: impl AsRef<MediaRange>) {
        let range = range.as_ref();
        if self.universal {
            return;
        }
        if range.is_wildcard_type() {
            *self = Self {
                slots: vec![Some(MediaRange::ANY)],
                len: 1,
                universal: true,
                types: HashMap::new(),
            };
            return;
        }
        let members = self
            .types
            .entry(range.top_level_type().to_owned())
            .or_default();
        if members.wildcard {
            return;
        }
        if range.is_wildcard_subtype() {
            for (_, slot) in members.subtypes.drain() {
                self.slots[slot] = None;
                self.len -= 1;
            }
            members.wildcard = true;
        } else if members.subtypes.contains_key(range.subtype()) {
            return;
        } else {
            members
                .subtypes
                .insert(range.subtype().to_owned(), self.slots.len());
        }
        self.slots.push(Some(range.without_parameters()));
        self.len += 1;
        // Each absorbed member leaves an empty slot. Compact once they outnumber the members, so iteration stays proportional to the members and the cost is amortized over the absorptions.
        if self.slots.len() - self.len > self.len {
            self.compact();
        }
    }

    /// Whether `media_type` is a member: a member range matches its type and subtype.
    // SECURITY: A client cannot satisfy an allowlist check by sending a wildcard such as `*/*`: this method accepts only a concrete `MediaType`, which parsing refuses to build from a wildcard. `matches` answers a different question, compatibility with a range, and is not an allowlist check.
    pub fn contains(&self, media_type: &MediaType) -> bool {
        self.matches(media_type)
    }

    /// Whether any member is compatible with `range`. For a concrete type this is membership, as with [`Self::contains`]; a `type/*` query matches when any member has that type, and `*/*` matches any non-empty set.
    pub fn matches(&self, range: impl AsRef<MediaRange>) -> bool {
        let range = range.as_ref();
        if self.universal {
            return true;
        }
        if range.is_wildcard_type() {
            return !self.is_empty();
        }
        let Some(members) = self.types.get(range.top_level_type()) else {
            return false;
        };
        members.wildcard
            || if range.is_wildcard_subtype() {
                !members.subtypes.is_empty()
            } else {
                members.subtypes.contains_key(range.subtype())
            }
    }

    /// Whether every media range is a member, because `*/*` was added.
    pub fn is_universal(&self) -> bool {
        self.universal
    }

    /// Whether the set has no members.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of members.
    pub fn len(&self) -> usize {
        self.len
    }

    /// The members, without parameters, in the order they were added: `*/*` alone for a universal set.
    pub fn iter(&self) -> MediaRangeSetIter<'_> {
        MediaRangeSetIter {
            slots: self.slots.iter(),
            remaining: self.len,
        }
    }

    /// Remove empty slots and record the members' new slots.
    fn compact(&mut self) {
        self.slots.retain(Option::is_some);
        for (slot, member) in self.slots.iter().flatten().enumerate() {
            let members = self
                .types
                .get_mut(member.top_level_type())
                .expect("every member's type is indexed");
            // A `type/*` member has no subtype slot to update.
            if let Some(index) = members.subtypes.get_mut(member.subtype()) {
                *index = slot;
            }
        }
    }
}

impl fmt::Debug for MediaRangeSet {
    /// Shows the members in the order they were added, without the index.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

/// An iterator over a [`MediaRangeSet`]'s members, in the order they were added.
#[derive(Clone, Debug)]
pub struct MediaRangeSetIter<'a> {
    slots: std::slice::Iter<'a, Option<MediaRange>>,
    remaining: usize,
}

impl<'a> Iterator for MediaRangeSetIter<'a> {
    type Item = &'a MediaRange;

    fn next(&mut self) -> Option<&'a MediaRange> {
        let member = self.slots.find_map(Option::as_ref)?;
        self.remaining -= 1;
        Some(member)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for MediaRangeSetIter<'_> {}

impl<'a> IntoIterator for &'a MediaRangeSet {
    type Item = &'a MediaRange;
    type IntoIter = MediaRangeSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for MediaRangeSet {
    type Item = MediaRange;
    type IntoIter = std::vec::IntoIter<MediaRange>;

    fn into_iter(self) -> Self::IntoIter {
        self.slots
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
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
