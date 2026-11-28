//! The radio group's walk over its items (`docs/design-system.md` § 5.8): the arrow keys skip the disabled items,
//! which Slint cannot do itself since its functions have no loops. It is the walk every group with one Tab stop
//! shares ([`crate::roving`]), so radios, segments and tabs move alike; `radio_nav!` wires it to an app's `RadioNav`.

pub use crate::roving::{edge, step};

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
