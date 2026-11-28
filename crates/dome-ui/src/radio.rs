//! The radio group's walk over its items (`docs/design-system.md` § 5.8): the arrow keys skip the disabled items,
//! which Slint cannot do itself since its functions have no loops. It is the segmented control's walk (§ 5.6) over the
//! items' `disabled` flags, so the two groups move alike; `radio_nav!` wires it to an app's `RadioNav` global.

pub use crate::segmented::{edge, step};

#[cfg(test)]
mod tests {
    use super::{edge, step};

    /// Three items, the middle one disabled.
    const MIDDLE_OFF: [bool; 3] = [false, true, false];

    #[test]
    fn the_arrows_skip_a_disabled_item_and_go_round() {
        assert_eq!(step(&MIDDLE_OFF, 0, true, true), 2);
        assert_eq!(step(&MIDDLE_OFF, 2, true, true), 0);
        assert_eq!(step(&MIDDLE_OFF, 2, true, false), 2);
        assert_eq!(edge(&MIDDLE_OFF, false), 0);
    }
}
