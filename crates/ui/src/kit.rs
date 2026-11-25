//! The Kit window (`docs/design-system.md` § 9): `cargo run -p diskdome-ui --features kit -- --kit` shows every
//! token and every component of the design system in every variant, size and state, and the review board. Only
//! with the `kit` feature, never in a release.

use slint::{ComponentHandle, Model};

use crate::generated::{KitIssueStatus, KitReview, KitWindow};

/// The flag that opens the Kit instead of the app.
pub const FLAG: &str = "--kit";

pub fn run() -> Result<(), slint::PlatformError> {
    open()?.run()
}

/// The Kit with its review board counted, not yet shown.
pub fn open() -> Result<KitWindow, slint::PlatformError> {
    let kit = KitWindow::new()?;
    count_review(&kit);
    Ok(kit)
}

/// Slint cannot count over an array: the top bar's «open» and «proposed» numbers come from here.
fn count_review(kit: &KitWindow) {
    let review = kit.global::<KitReview>();
    let issues = review.get_issues();
    let count =
        |status: KitIssueStatus| issues.iter().filter(|issue| issue.status == status).count();
    review.set_open_count(to_int(count(KitIssueStatus::Open)));
    review.set_proposed_count(to_int(count(KitIssueStatus::Proposed)));
}

/// A handful of issues always fits an `i32`; saturate rather than panic if it ever did not.
fn to_int(count: usize) -> i32 {
    i32::try_from(count).unwrap_or(i32::MAX)
}
