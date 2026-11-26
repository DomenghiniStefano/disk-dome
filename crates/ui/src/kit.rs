//! The Kit window (`docs/design-system.md` § 9): `cargo run -p diskdome-ui --features kit -- --kit` shows every
//! token and every component of the design system in every variant, size and state, and the review board. Only
//! with the `kit` feature, never in a release.

use slint::{ComponentHandle, Model};

use regex::RegexBuilder;

use crate::generated::{KitIssueStatus, KitReview, KitSearch, KitWindow};

/// The flag that opens the Kit instead of the app.
pub const FLAG: &str = "--kit";

pub fn run() -> Result<(), slint::PlatformError> {
    open()?.run()
}

/// The Kit with its review board counted, not yet shown.
pub fn open() -> Result<KitWindow, slint::PlatformError> {
    let kit = KitWindow::new()?;
    crate::design_system::install(&kit);
    count_review(&kit);
    let search = kit.global::<KitSearch>();
    search.on_matches(|title, query, match_case, regex| {
        section_matches(&title, &query, match_case, regex)
    });
    search.on_valid(|query, regex| pattern(&query, true, regex).is_ok());
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

/// The search as a regular expression; plain text is escaped first, so both go through one matcher.
fn pattern(query: &str, match_case: bool, regex: bool) -> Result<regex::Regex, regex::Error> {
    let source = if regex {
        query.to_owned()
    } else {
        regex::escape(query)
    };
    RegexBuilder::new(&source)
        .case_insensitive(!match_case)
        .build()
}

/// Whether a section title matches the Kit's search; an invalid pattern matches nothing.
fn section_matches(title: &str, query: &str, match_case: bool, regex: bool) -> bool {
    pattern(query, match_case, regex).is_ok_and(|re| re.is_match(title))
}

#[cfg(test)]
mod tests {
    use super::section_matches;

    #[test]
    fn plain_text_matches_anywhere_ignoring_case() {
        assert!(section_matches(
            "Button · variants × states",
            "variants",
            false,
            false
        ));
        assert!(section_matches(
            "Button · variants × states",
            "BUTTON",
            false,
            false
        ));
        assert!(!section_matches(
            "Button · variants × states",
            "BUTTON",
            true,
            false
        ));
    }

    #[test]
    fn plain_text_is_not_a_pattern() {
        assert!(!section_matches("Colour · accent", "c.lour", false, false));
        assert!(section_matches("Colour · accent", "c.lour", false, true));
    }

    #[test]
    fn an_invalid_pattern_matches_nothing() {
        assert!(!section_matches("Colour · accent", "([a-z", false, true));
    }
}
