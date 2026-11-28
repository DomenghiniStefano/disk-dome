//! The segmented control's walk over its segments (`docs/design-system.md` § 5.6): the arrow keys, Home and End skip
//! the disabled segments. The walk is the one every row with a single Tab stop shares, [`crate::roving`];
//! `segmented_nav!` wires it to an app's `SegmentedNav` global.

pub use crate::roving::{edge, step};
