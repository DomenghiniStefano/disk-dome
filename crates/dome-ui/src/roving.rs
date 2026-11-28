//! The walk over a row of items with one Tab stop (roving focus), shared by the segmented control (§ 5.6) and the
//! tabs (§ 5.7): the arrow keys, Home and End skip the disabled items, which Slint cannot do itself since its functions
//! have no loops. Pure functions over the items' `disabled` flags; `segmented_nav!` and `tab_nav!` wire them to an
//! app's `SegmentedNav` and `TabNav` globals.

/// The indices of the items the keyboard can choose, in order.
fn enabled(disabled: &[bool]) -> Vec<i32> {
    disabled
        .iter()
        .enumerate()
        .filter(|&(_, &off)| !off)
        .filter_map(|(index, _)| i32::try_from(index).ok())
        .collect()
}

/// The first or the last enabled item; -1 with none.
pub fn edge(disabled: &[bool], last: bool) -> i32 {
    let enabled = enabled(disabled);
    let found = if last {
        enabled.last()
    } else {
        enabled.first()
    };
    found.copied().unwrap_or(-1)
}

/// The nearest enabled item after `from` (`forward`) or before it. At the end, `wrap` goes round to the other end;
/// otherwise the choice stays where it is, or, on an item disabled after it was chosen, on the nearest enabled one.
/// With nothing chosen (`from` out of range), the first enabled item going forward and the last going back.
/// -1 with no enabled item.
pub fn step(disabled: &[bool], from: i32, forward: bool, wrap: bool) -> i32 {
    let enabled = enabled(disabled);
    let (Some(&first), Some(&last)) = (enabled.first(), enabled.last()) else {
        return -1;
    };
    let in_range = usize::try_from(from).is_ok_and(|index| index < disabled.len());
    if !in_range {
        return if forward { first } else { last };
    }
    let next = if forward {
        enabled.iter().copied().find(|&index| index > from)
    } else {
        enabled.iter().rev().copied().find(|&index| index < from)
    };
    let other_end = if forward { first } else { last };
    let this_end = if forward { last } else { first };
    match next {
        Some(index) => index,
        None if wrap => other_end,
        None if enabled.contains(&from) => from,
        None => this_end,
    }
}

#[cfg(test)]
mod tests {
    use super::{edge, step};

    /// Four items, the second disabled.
    const SECOND_OFF: [bool; 4] = [false, true, false, false];

    #[test]
    fn arrows_skip_a_disabled_item() {
        assert_eq!(step(&SECOND_OFF, 0, true, false), 2);
        assert_eq!(step(&SECOND_OFF, 2, false, false), 0);
    }

    #[test]
    fn the_ends_stop_or_wrap() {
        assert_eq!(step(&SECOND_OFF, 3, true, false), 3);
        assert_eq!(step(&SECOND_OFF, 3, true, true), 0);
        assert_eq!(step(&SECOND_OFF, 0, false, false), 0);
        assert_eq!(step(&SECOND_OFF, 0, false, true), 3);
    }

    #[test]
    fn with_no_valid_choice_the_arrows_start_at_an_end() {
        assert_eq!(step(&SECOND_OFF, -1, true, false), 0);
        assert_eq!(step(&SECOND_OFF, 9, false, false), 3);
        // The chosen item was disabled after it was chosen: the arrow goes on from it.
        assert_eq!(step(&SECOND_OFF, 1, true, false), 2);
        assert_eq!(step(&SECOND_OFF, 1, false, false), 0);
        // ... and with nothing beyond it, back to the nearest enabled one.
        assert_eq!(step(&[false, true], 1, true, false), 0);
    }

    #[test]
    fn nothing_enabled_gives_no_item() {
        assert_eq!(step(&[true, true], 0, true, true), -1);
        assert_eq!(edge(&[true, true], false), -1);
        assert_eq!(edge(&[], true), -1);
    }

    #[test]
    fn home_and_end_are_the_enabled_ends() {
        assert_eq!(edge(&[true, false, false, true], false), 1);
        assert_eq!(edge(&[true, false, false, true], true), 2);
    }
}
