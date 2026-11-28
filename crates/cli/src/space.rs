//! A volume's space as Windows reports it (`docs/design-system.md` § 4.4, § 12 test 5): `space PATH` prints it, and a
//! scan of a drive checks that what it found plus Windows' free space adds up to the capacity, within 1 %.

use diskdome_platform::{VolumeSpace, volume_space};

/// § 12 test 5: used + free = capacity within 1 %.
const TOLERANCE_PERCENT: f64 = 1.0;

const GB: f64 = 1e9;

/// How far used + free is from the capacity, in percent of the capacity (positive: more than the capacity).
pub fn gap_percent(used: u64, free: u64, capacity: u64) -> f64 {
    if capacity == 0 {
        return 0.0;
    }
    (used as f64 + free as f64 - capacity as f64) / capacity as f64 * 100.0
}

fn lines(space: &VolumeSpace) -> String {
    let mut out = format!(
        "capacity {:.3} GB, free {:.3} GB, free for this user {:.3} GB",
        space.capacity as f64 / GB,
        space.free as f64 / GB,
        space.free_to_user as f64 / GB
    );
    if space.quota_limited() {
        out.push_str("\nquota: this user may write less than the volume has free (the tooltip says so, § 4.4)");
    }
    out
}

/// `space PATH`: what Windows reports for the volume holding `PATH` (a drive, a folder, a network share).
pub fn run(path: &str) -> Result<(), String> {
    let space = volume_space(path).ok_or_else(|| format!("Windows reports no space for {path}"))?;
    println!("{path}: {}", lines(&space));
    Ok(())
}

/// After a scan of `drive`: the bytes the scan found on it against Windows' free space and capacity.
pub fn check_scan(drive: char, used: u64) {
    let root = format!("{drive}:\\");
    let Some(space) = volume_space(&root) else {
        println!("space: Windows reports no space for {root}");
        return;
    };
    let gap = gap_percent(used, space.free, space.capacity);
    println!("space: {}", lines(&space));
    println!(
        "space: scanned {:.3} GB + free {:.3} GB = {:.3} GB, {gap:+.2} % of the capacity ({})",
        used as f64 / GB,
        space.free as f64 / GB,
        (used + space.free) as f64 / GB,
        if gap.abs() <= TOLERANCE_PERCENT {
            "within 1 %"
        } else {
            "OUTSIDE 1 %"
        }
    );
}

#[cfg(test)]
mod tests {
    use super::gap_percent;

    #[test]
    fn the_gap_is_a_share_of_the_capacity() {
        assert_eq!(gap_percent(60, 40, 100), 0.0);
        assert_eq!(gap_percent(59, 40, 100), -1.0);
        assert_eq!(gap_percent(62, 40, 100), 2.0);
        assert_eq!(gap_percent(5, 5, 0), 0.0, "an empty capacity says nothing");
    }
}
