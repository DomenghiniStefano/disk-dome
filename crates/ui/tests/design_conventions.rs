//! The design system's rules on the `.slint` sources (`docs/design-system.md` § 10), read as text.
//!
//! Every rule runs on the real sources and on its fixtures in `tests/fixtures/conventions/`: each `bad_<rule>.slint`
//! must break that rule, `good.slint` must break none. The fixtures are checked as if they sat at the path written in
//! their first line (`// as: <path relative to crates/ui/ui>`), so the rules' «where» is tested too.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// Throwaway spikes, not checked: the step-3 proof table (card 3c) and the title bar test (card 4.T1).
const EXEMPT: &[&str] = &["proof.slint", "spike/titlebar.slint"];

struct Rule {
    name: &'static str,
    check: fn(&Source) -> Vec<String>,
}

struct Source {
    /// Path relative to `crates/ui/ui`, with `/`.
    path: String,
    /// The text with comments blanked out.
    code: String,
    /// `code` with the contents of string literals blanked out too.
    bare: String,
}

impl Source {
    fn new(path: &str, text: &str) -> Source {
        let code = strip_comments(text);
        let bare = Regex::new(r#""(?:[^"\\]|\\.)*""#)
            .expect("test data is valid")
            .replace_all(&code, "\"\"")
            .into_owned();
        Source {
            path: path.to_owned(),
            code,
            bare,
        }
    }
    fn in_tokens(&self) -> bool {
        self.path.starts_with("design-system/tokens/")
    }
    fn is_screen(&self) -> bool {
        self.path.starts_with("screens/") || self.path == "app.slint"
    }
}

fn strip_comments(text: &str) -> String {
    let block = Regex::new(r"(?s)/\*.*?\*/").expect("test data is valid");
    let text = block.replace_all(text, |c: &regex::Captures| {
        c[0].chars()
            .map(|ch| if ch == '\n' { '\n' } else { ' ' })
            .collect::<String>()
    });
    Regex::new(r"(?m)//.*$")
        .expect("test data is valid")
        .replace_all(&text, "")
        .into_owned()
}

fn hits(re: &str, text: &str) -> Vec<String> {
    Regex::new(re)
        .expect("test data is valid")
        .find_iter(text)
        .map(|m| m.as_str().to_owned())
        .collect()
}

const RULES: &[Rule] = &[
    Rule {
        name: "literal-colour",
        check: |s| {
            if s.in_tokens() {
                return vec![];
            }
            hits(
                r"#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3,4})\b|\brgba?\(|\bhsla?\(|\bColors\.",
                &s.bare,
            )
        },
    },
    Rule {
        name: "literal-length",
        check: |s| {
            if s.in_tokens() {
                return vec![];
            }
            hits(r"\b\d*\.?\d+(?:px|phx|rem|pt|cm|mm|in)\b", &s.bare)
                .into_iter()
                .filter(|h| {
                    h.trim_start_matches(['0', '.'])
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_digit())
                })
                .collect()
        },
    },
    Rule {
        name: "literal-duration",
        check: |s| {
            if s.in_tokens() {
                return vec![];
            }
            hits(r"\b\d*\.?\d+(?:ms|s)\b", &s.bare)
        },
    },
    Rule {
        name: "font-literal",
        check: |s| {
            if s.in_tokens() || s.path == "design-system/components/label.slint" {
                return vec![];
            }
            hits(
                r"\b(?:font-size|font-weight|font-family|letter-spacing)\s*:",
                &s.bare,
            )
        },
    },
    Rule {
        name: "std-widgets",
        check: |s| {
            if s.path.starts_with("design-system/components/") {
                return vec![];
            }
            hits(r#"from\s+"std-widgets\.slint""#, &s.code)
        },
    },
    Rule {
        name: "boundary",
        check: |s| {
            if !s.is_screen() {
                return vec![];
            }
            hits(r#"from\s+"[^"]*design-system/[^"]*""#, &s.code)
                .into_iter()
                .filter(|h| !h.ends_with("design-system/index.slint\""))
                .collect()
        },
    },
    Rule {
        name: "raw-element",
        check: |s| {
            if !s.is_screen() {
                return vec![];
            }
            hits(
                r"\b(?:Text|TextInput|TouchArea|FocusScope|Image|PopupWindow|Flickable)\s*(?:\{|inherits)|inherits\s+(?:Text|TextInput|TouchArea|FocusScope|Image|PopupWindow|Flickable)\b",
                &s.bare,
            )
        },
    },
    Rule {
        name: "untranslated",
        check: |s| {
            if s.path.starts_with("design-system/kit/") {
                return vec![];
            }
            hits(r#"\btext\s*:\s*"(?:[^"\\]|\\.)+""#, &s.code)
        },
    },
    Rule {
        name: "icon-label",
        check: |s| {
            // Blind spot: an icon-only `Button` is not recognised; only `IconButton` is checked.
            let mut found = vec![];
            for m in Regex::new(r"\bIconButton\s*\{")
                .expect("test data is valid")
                .find_iter(&s.bare)
            {
                let body = block_after(&s.bare, m.end());
                if !body.contains("accessible-label") {
                    found.push(m.as_str().to_owned());
                }
            }
            found
        },
    },
    Rule {
        name: "binary-units",
        check: |s| {
            if s.in_tokens() {
                return vec![];
            }
            hits(r#""[^"]*\b(?:GiB|MiB|TiB)\b[^"]*""#, &s.code)
        },
    },
];

/// The text of the `{ … }` block that opens just before `start`.
fn block_after(text: &str, start: usize) -> &str {
    let mut depth = 1usize;
    for (i, c) in text[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[start..start + i];
                }
            }
            _ => {}
        }
    }
    &text[start..]
}

fn ui_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn slint_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("test data is valid") {
        let path = entry.expect("test data is valid").path();
        if path.is_dir() {
            slint_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "slint") {
            out.push(path);
        }
    }
}

fn violations(source: &Source) -> Vec<String> {
    RULES
        .iter()
        .flat_map(|rule| {
            (rule.check)(source)
                .into_iter()
                .map(move |hit| format!("{}: {} `{}`", source.path, rule.name, hit))
        })
        .collect()
}

#[test]
fn sources_follow_the_rules() {
    let root = ui_dir();
    let mut files = vec![];
    slint_files(&root, &mut files);
    let mut found = vec![];
    let mut sources = vec![];
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("test data is valid")
            .to_string_lossy()
            .replace('\\', "/");
        if EXEMPT.contains(&rel.as_str()) {
            continue;
        }
        let source = Source::new(
            &rel,
            &fs::read_to_string(&file).expect("test data is valid"),
        );
        found.extend(violations(&source));
        sources.push(source);
    }
    found.extend(kit_coverage(&sources));
    assert!(
        found.is_empty(),
        "design rules broken:\n{}",
        found.join("\n")
    );
}

/// `kit-coverage`: every component exported by `design-system/components/` appears in `kit/kit.slint`.
fn kit_coverage(sources: &[Source]) -> Vec<String> {
    let kit = sources
        .iter()
        .find(|s| s.path == "design-system/kit/kit.slint");
    let export = Regex::new(r"export\s+component\s+([A-Za-z][\w-]*)").expect("test data is valid");
    sources
        .iter()
        .filter(|s| s.path.starts_with("design-system/components/"))
        .flat_map(|s| {
            export
                .captures_iter(&s.code)
                .map(|c| c[1].to_owned())
                .collect::<Vec<_>>()
        })
        .filter(|name| {
            kit.is_none_or(|k| {
                !Regex::new(&format!(r"\b{name}\b"))
                    .expect("test data is valid")
                    .is_match(&k.code)
            })
        })
        .map(|name| format!("design-system/kit/kit.slint: kit-coverage `{name}` missing"))
        .collect()
}

fn fixture(name: &str) -> Source {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/conventions")
        .join(name);
    let text = fs::read_to_string(&path).expect("test data is valid");
    let first = text.lines().next().expect("test data is valid");
    let as_path = first
        .strip_prefix("// as: ")
        .expect("a fixture starts with `// as: <path>`");
    Source::new(as_path.trim(), &text)
}

#[test]
fn every_rule_catches_its_bad_fixture() {
    for rule in RULES {
        let source = fixture(&format!("bad_{}.slint", rule.name));
        assert!(
            !(rule.check)(&source).is_empty(),
            "rule {} missed its bad fixture",
            rule.name
        );
    }
}

#[test]
fn the_good_fixture_breaks_no_rule() {
    let source = fixture("good.slint");
    assert_eq!(violations(&source), Vec::<String>::new());
}

#[test]
fn kit_coverage_catches_a_missing_component() {
    let component = Source::new(
        "design-system/components/chip.slint",
        "export component Chip {}",
    );
    let kit = Source::new(
        "design-system/kit/kit.slint",
        "export component KitWindow { Rectangle {} }",
    );
    assert_eq!(kit_coverage(&[component, kit]).len(), 1);
}
