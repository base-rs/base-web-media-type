// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use crate::{MediaRange, MediaType};

/// A set of media ranges, such as the types an upload field accepts.
///
/// Membership depends only on types and subtypes: members are stored without parameters. Ranges covered by a wildcard are absorbed: adding `image/*` replaces `image/png`, and adding `*/*` replaces every member, making the set universal.
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
    /// Members without parameters, in the order they were added, none covered by another.
    ranges: Vec<MediaRange>,
}

impl MediaRangeSet {
    /// An empty set. Build one from ranges with `collect` or `extend`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a range, without its parameters, unless a member already covers it. Members it covers are removed.
    pub fn insert(&mut self, range: impl AsRef<MediaRange>) {
        let range = range.as_ref();
        if self.ranges.iter().any(|member| covers(member, range)) {
            return;
        }
        self.ranges.retain(|member| !covers(range, member));
        self.ranges.push(range.without_parameters());
    }

    /// Whether `media_type` is a member: a member range matches its type and subtype.
    pub fn contains(&self, media_type: &MediaType) -> bool {
        self.matches(media_type)
    }

    /// Whether any member is compatible with `range`. For a concrete type this is membership, as with [`Self::contains`]; a `type/*` query matches when any member has that type, and `*/*` matches any non-empty set.
    pub fn matches(&self, range: impl AsRef<MediaRange>) -> bool {
        let range = range.as_ref();
        self.ranges.iter().any(|member| member.matches(range))
    }

    /// Whether every media range is a member, because `*/*` was added.
    pub fn is_universal(&self) -> bool {
        self.ranges
            .first()
            .is_some_and(|member| member.is_wildcard_type())
    }

    /// Whether the set has no members.
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// The number of members.
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// The members, without parameters, in the order they were added: `*/*` alone for a universal set.
    pub fn iter(&self) -> std::slice::Iter<'_, MediaRange> {
        self.ranges.iter()
    }
}

/// Whether `member` already includes every type `range` names.
fn covers(member: &MediaRange, range: &MediaRange) -> bool {
    member.is_wildcard_type()
        || (member.top_level_type() == range.top_level_type()
            && (member.is_wildcard_subtype() || member.subtype() == range.subtype()))
}

impl<'a> IntoIterator for &'a MediaRangeSet {
    type Item = &'a MediaRange;
    type IntoIter = std::slice::Iter<'a, MediaRange>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for MediaRangeSet {
    type Item = MediaRange;
    type IntoIter = std::vec::IntoIter<MediaRange>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
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
