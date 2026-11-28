//! The breadcrumb's fit (`docs/design-system.md` § 5.13): which segments fold into «…» when the path does not fit its
//! width, which Slint cannot work out itself since its functions have no loops. Pure functions over the segments'
//! natural widths, measured by the Slint half; `breadcrumb_fit!` wires them to an app's `BreadcrumbFit` global.

use std::ops::Range;

/// Lengths from Slint's layout are floats: a path this close to its width still fits.
const FIT_TOLERANCE: f32 = 0.01;

/// The first `count` widths, those missing (not yet measured) as 0.
fn padded(widths: &[f32], count: usize) -> Vec<f32> {
    (0..count)
        .map(|index| widths.get(index).copied().unwrap_or(0.0))
        .collect()
}

/// `widths` with the one at `index` set to `width`, grown with zeros to reach it. The Slint half keeps its widths with
/// it, one segment at a time, as each one is measured.
pub fn with_width(widths: &[f32], index: i32, width: f32) -> Vec<f32> {
    let mut out = widths.to_vec();
    if let Ok(index) = usize::try_from(index) {
        if out.len() <= index {
            out.resize(index + 1, 0.0);
        }
        out[index] = width;
    }
    out
}

/// The whole path's width, every segment shown: the breadcrumb's preferred width.
pub fn total(widths: &[f32], count: usize) -> f32 {
    padded(widths, count).iter().sum()
}

/// The first segment shown after the fold. 0 when the whole path fits. Otherwise the fold takes the fewest segments
/// so that `ellipsis` (the «…» with its separator) and the segments from the returned one to the end fit in
/// `available`, after the first segment when `keep_first` (the fold in the middle) or alone (the fold from the
/// start). When even the last segment alone does not fit, the last one: it is then cut with «…».
///
/// Each width is a segment with the separator before it (none before the first).
pub fn first_shown(
    widths: &[f32],
    count: usize,
    available: f32,
    ellipsis: f32,
    keep_first: bool,
) -> i32 {
    let widths = padded(widths, count);
    let fits = |width: f32| width <= available + FIT_TOLERANCE;
    if fits(widths.iter().sum()) {
        return 0;
    }
    let head = if keep_first {
        widths.first().copied().unwrap_or(0.0)
    } else {
        0.0
    };
    let last = count.saturating_sub(1);
    let first = (1..last)
        .find(|&from| fits(head + ellipsis + widths[from..].iter().sum::<f32>()))
        .unwrap_or(last);
    i32::try_from(first).unwrap_or(i32::MAX)
}

/// The segments folded into «…», in order: after the first one with the fold in the middle (`keep_first`), from the
/// start otherwise. Empty when nothing is folded.
pub fn hidden(first_shown: i32, count: usize, keep_first: bool) -> Range<usize> {
    let start = usize::from(keep_first);
    let end = usize::try_from(first_shown).unwrap_or(0).min(count);
    start..end.max(start)
}

#[cfg(test)]
mod tests {
    use super::{first_shown, hidden, total, with_width};

    /// Five segments: «C:» 30, then four folders 60 wide each with its separator.
    const PATH: [f32; 5] = [30.0, 60.0, 60.0, 60.0, 60.0];
    /// The «…» with the separator before it.
    const ELLIPSIS: f32 = 36.0;

    #[test]
    fn a_path_that_fits_folds_nothing() {
        assert_eq!(total(&PATH, 5), 270.0);
        assert_eq!(first_shown(&PATH, 5, 270.0, ELLIPSIS, true), 0);
        assert_eq!(first_shown(&PATH, 5, 400.0, ELLIPSIS, false), 0);
        assert!(hidden(0, 5, true).is_empty());
        assert!(hidden(0, 5, false).is_empty());
    }

    #[test]
    fn the_middle_folds_the_fewest_segments_and_keeps_the_first() {
        // 30 + 36 + 60 + 60 + 60 = 246: one folder folds.
        assert_eq!(first_shown(&PATH, 5, 250.0, ELLIPSIS, true), 2);
        assert_eq!(hidden(2, 5, true), 1..2);
        // 30 + 36 + 60 + 60 = 186: two fold.
        assert_eq!(first_shown(&PATH, 5, 200.0, ELLIPSIS, true), 3);
        assert_eq!(hidden(3, 5, true), 1..3);
    }

    #[test]
    fn the_start_folds_the_first_segment_too() {
        // 36 + 60 * 3 = 216: «C:» and one folder fold.
        assert_eq!(first_shown(&PATH, 5, 250.0, ELLIPSIS, false), 2);
        assert_eq!(hidden(2, 5, false), 0..2);
    }

    #[test]
    fn the_last_segment_always_stays() {
        // Not even the last one fits after the fold: it stays, to be cut.
        assert_eq!(first_shown(&PATH, 5, 50.0, ELLIPSIS, true), 4);
        assert_eq!(hidden(4, 5, true), 1..4);
        // Two segments in the middle mode: nothing to fold between them.
        assert_eq!(first_shown(&PATH[..2], 2, 50.0, ELLIPSIS, true), 1);
        assert!(hidden(1, 2, true).is_empty());
        // One segment: nothing folds, it is cut.
        assert_eq!(first_shown(&PATH[..1], 1, 10.0, ELLIPSIS, false), 0);
        assert_eq!(first_shown(&[], 0, 10.0, ELLIPSIS, true), 0);
    }

    #[test]
    fn widths_not_yet_measured_count_as_nothing() {
        assert_eq!(total(&PATH[..2], 5), 90.0);
        assert_eq!(first_shown(&PATH[..2], 5, 90.0, ELLIPSIS, true), 0);
    }

    #[test]
    fn a_width_is_kept_at_its_index() {
        assert_eq!(with_width(&[], 2, 5.0), vec![0.0, 0.0, 5.0]);
        assert_eq!(with_width(&[1.0, 2.0], 0, 7.0), vec![7.0, 2.0]);
        assert_eq!(with_width(&[1.0], -1, 7.0), vec![1.0]);
    }
}
