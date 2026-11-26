//! The token values in `crates/dome-ui/ui/tokens/` against the tables of `docs/design-system.md` (§ 3, § 4.7):
//! every colour, length, opacity and duration the spec gives a token name exists with the same value.
//!
//! Blind spot: the type steps (`Type`, § 3.3) are pure functions and are checked in the Kit, not here.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use regex::Regex;

fn read(rel: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).expect("test data is valid")
}

fn token(rel: &str) -> String {
    read(&format!("../dome-ui/ui/tokens/{rel}"))
}

/// The table rows of the spec's § 3 plus the chart colours of § 4.7, as cells without backticks.
fn spec_rows() -> Vec<Vec<String>> {
    let doc = read("../../docs/design-system.md");
    let start = doc.find("## 3. Tokens").expect("test data is valid");
    let end = doc
        .find("## 4. Showing the data")
        .expect("test data is valid");
    doc.lines()
        .filter(|l| l.starts_with('|'))
        .filter(|l| {
            doc[start..end].contains(*l) || l.starts_with("| `chart-") || l.starts_with("| `delta-")
        })
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().replace('`', ""))
                .collect()
        })
        .collect()
}

fn hex(cell: &str) -> Option<String> {
    Regex::new(r"^#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?\b")
        .expect("test data is valid")
        .find(cell)
        .map(|m| m.as_str().to_lowercase())
}

fn number(cell: &str) -> Option<String> {
    Regex::new(r"^(-?\d+(?:\.\d+)?)(?: ?ms)?$")
        .expect("test data is valid")
        .captures(cell)
        .map(|c| c[1].trim_end_matches(".0").to_owned())
}

fn is_name(cell: &str) -> bool {
    Regex::new(r"^[a-z][a-z0-9-]*$")
        .expect("test data is valid")
        .is_match(cell)
}

fn normalise(n: &str) -> String {
    let v: f64 = n.parse().expect("test data is valid");
    format!("{v}")
}

#[test]
fn theme_colours_match_the_spec() {
    let theme = token("theme.slint");
    let re = Regex::new(r"out property <color> ([\w-]+): dark \? (#\w+) : (#\w+);")
        .expect("test data is valid");
    let slint: BTreeMap<String, (String, String)> = re
        .captures_iter(&theme)
        .map(|c| (c[1].to_owned(), (c[2].to_lowercase(), c[3].to_lowercase())))
        .collect();

    let mut expected: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut accent: BTreeMap<String, [String; 2]> = BTreeMap::new();
    for row in spec_rows() {
        if row.len() >= 3
            && is_name(&row[0])
            && let (Some(dark), Some(light)) = (hex(&row[1]), hex(&row[2]))
        {
            expected.insert(row[0].clone(), (dark, light));
        }
        // The accent table: one row per theme, the roles as columns.
        if row.len() >= 9 && row[0].starts_with("Laguna") {
            let theme = usize::from(row[1] == "light");
            let roles = [
                "accent",
                "accent-hover",
                "accent-pressed",
                "on-accent",
                "accent-text",
                "accent-soft",
                "focus",
            ];
            for (i, role) in roles.iter().enumerate() {
                accent.entry((*role).to_owned()).or_default()[theme] =
                    hex(&row[2 + i]).expect("test data is valid");
            }
        }
    }
    for (role, [dark, light]) in accent {
        expected.insert(role, (dark, light));
    }
    assert!(
        expected.len() >= 37,
        "the spec's colour tables were not found"
    );
    let mismatches: Vec<String> = expected
        .iter()
        .filter(|(name, values)| slint.get(*name) != Some(values))
        .map(|(name, values)| format!("{name}: spec {values:?}, Slint {:?}", slint.get(name)))
        .collect();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn sizes_opacity_and_motion_match_the_spec() {
    let size = token("size.slint") + &token("motion.slint");
    let re = Regex::new(
        r"out property <(?:length|float|duration)> ([\w-]+): (?:(?:compact|reduced) \? (-?[\d.]+)(?:px|ms) : )?(-?[\d.]+)(?:px|ms)?;",
    )
    .expect("test data is valid");
    // name -> (normal, compact); for motion the «compact» slot holds the reduced value and is not compared.
    let slint: BTreeMap<String, (String, Option<String>)> = re
        .captures_iter(&size)
        .map(|c| {
            let other = c.get(2).map(|m| normalise(m.as_str()));
            (c[1].to_owned(), (normalise(&c[3]), other))
        })
        .collect();

    let mut checked = 0;
    let mut mismatches = vec![];
    for row in spec_rows() {
        if row.len() < 2 || !is_name(&row[0]) {
            continue;
        }
        let Some(normal) = number(&row[1]) else {
            continue;
        };
        let compact = row.get(2).and_then(|c| number(c));
        let name = &row[0];
        checked += 1;
        match slint.get(name) {
            Some((n, c)) if *n == normalise(&normal) => {
                if let Some(compact) = compact
                    && c.as_deref() != Some(normalise(&compact).as_str())
                {
                    mismatches.push(format!("{name}: compact spec {compact}, Slint {c:?}"));
                }
            }
            other => mismatches.push(format!("{name}: spec {normal}, Slint {other:?}")),
        }
    }
    assert!(
        checked > 60,
        "the spec's size tables were not found ({checked} rows)"
    );
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn spacing_grid_names_say_their_value() {
    let size = token("size.slint");
    for c in Regex::new(r"space-(\d+): (\d+)px;")
        .expect("test data is valid")
        .captures_iter(&size)
    {
        assert_eq!(&c[1], &c[2]);
    }
    let doc = read("../../docs/design-system.md");
    let grid = Regex::new(r"Grid: `space-([\d ]+)`")
        .expect("test data is valid")
        .captures(&doc)
        .expect("test data is valid")[1]
        .to_owned();
    for step in grid.split(' ').filter(|s| *s != "0") {
        assert!(
            size.contains(&format!("space-{step}: {step}px;")),
            "space-{step} missing"
        );
    }
}

#[test]
fn compact_is_never_larger_than_normal() {
    let size = token("size.slint");
    for c in Regex::new(r"([\w-]+): compact \? (\d+)px : (\d+)px;")
        .expect("test data is valid")
        .captures_iter(&size)
    {
        let (compact, normal): (u32, u32) = (
            c[2].parse().expect("test data is valid"),
            c[3].parse().expect("test data is valid"),
        );
        assert!(
            compact <= normal,
            "{}: compact {compact} > normal {normal}",
            &c[1]
        );
    }
    assert!(
        size.contains("row: compact ? 18px"),
        "row is at least 18 px"
    );
    assert!(
        size.contains("control: compact ? 24px"),
        "control is at least 20 px"
    );
}

#[test]
fn data_colours_match_the_spec() {
    let data = token("data.slint");
    let doc = read("../../docs/design-system.md");
    // | Video / Video | `video` | `#d9704f` | `#f18664` | `#c15a3a` | …
    let row =
        Regex::new(r"(?m)^\| [^|]+ \| `([a-z]+)` \| `(#\w{6})` \| `(#\w{6})` \| `(#\w{6})` \|")
            .expect("test data is valid");
    let mut n = 0;
    for c in row.captures_iter(&doc) {
        let (category, base, lighter, darker) = (&c[1], &c[2], &c[3], &c[4]);
        let line = format!("lighter ? {lighter} : shade == DataShade.darker ? {darker} : {base};");
        assert!(data.contains(&line), "{category}: expected `{line}`");
        if category != "other" {
            assert!(
                data.contains(&format!("FileCategory.{category} {{")),
                "{category} has no branch"
            );
        }
        n += 1;
    }
    assert_eq!(n, 12, "twelve categories in the spec");
    let age = Regex::new(r"(?m)^\| `age-[a-z0-9]+` \| [^|]+ \| `(#\w{6})` \|")
        .expect("test data is valid");
    let ages: Vec<String> = age.captures_iter(&doc).map(|c| c[1].to_owned()).collect();
    assert_eq!(ages.len(), 6);
    for colour in ages {
        assert!(data.contains(&colour), "age colour {colour} missing");
    }
}
