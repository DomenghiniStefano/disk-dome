//! The Italian translation (`lang/it/LC_MESSAGES/diskdome-ui.po`, bundled by `build.rs`) holds exactly the
//! `@tr` strings of the `.slint` sources, every one translated. `./scripts/translations.ps1` adds new strings.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use diskdome_ui::generated::MainWindow;
use diskdome_ui::headless::Headless;
use regex::Regex;
use slint::SelectBundledTranslationError;

/// One translatable string: the context (the enclosing component unless `@tr("context" => …)`) and the text.
type Key = (String, String);

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn slint_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("test data is valid")
        .map(|entry| entry.expect("test data is valid").path())
        .flat_map(|path| {
            if path.is_dir() {
                slint_files(&path)
            } else if path.extension().is_some_and(|e| e == "slint") {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// The `@tr` keys of one source. A string without a context takes the nearest `component` or `global` above
/// it, as the Slint compiler does.
fn source_keys(source: &str) -> Vec<Key> {
    let string = r#""((?:[^"\\]|\\.)*)""#;
    let tr =
        Regex::new(&format!(r"@tr\(\s*(?:{string}\s*=>\s*)?{string}")).expect("test data is valid");
    let owner = Regex::new(r"\b(?:component|global)\s+([\w-]+)").expect("test data is valid");
    tr.captures_iter(source)
        .map(|c| {
            let start = c.get(0).expect("whole match").start();
            let context = match c.get(1) {
                Some(context) => context.as_str().to_owned(),
                None => owner
                    .captures_iter(&source[..start])
                    .last()
                    .map(|o| o[1].to_owned())
                    .expect("an @tr sits inside a component or a global"),
            };
            (context, unescape(&c[2]))
        })
        .collect()
}

fn unescape(text: &str) -> String {
    text.replace("\\\"", "\"").replace("\\\\", "\\")
}

/// The app's sources and the design system's (`@dome-ui`): both are compiled into the app, with its translation.
fn sources() -> BTreeSet<Key> {
    [manifest().join("ui"), manifest().join("../dome-ui/ui")]
        .iter()
        .flat_map(|dir| slint_files(dir))
        .flat_map(|path| source_keys(&fs::read_to_string(path).expect("test data is valid")))
        .collect()
}

/// One `.po` entry: its key, its translations (one, or one per plural form) and whether it is marked fuzzy.
struct Entry {
    key: Key,
    translations: Vec<String>,
    fuzzy: bool,
}

/// A small `.po` reader: entries separated by blank lines, strings that continue on the next lines.
fn po_entries(po: &str) -> Vec<Entry> {
    po.split("\n\n")
        .filter_map(|block| {
            let mut fields: Vec<(String, String)> = Vec::new();
            for line in block.lines() {
                match line.strip_prefix('"') {
                    Some(rest) => {
                        let (_, value) = fields
                            .last_mut()
                            .expect("a continued string follows a field");
                        value.push_str(&unescape(rest.trim_end_matches('"')));
                    }
                    None if line.starts_with('#') => {}
                    None => {
                        let (name, value) = line.split_once(' ').expect("a field has a value");
                        let value = value.trim_matches('"');
                        fields.push((name.to_owned(), unescape(value)));
                    }
                }
            }
            let field = |name: &str| {
                fields
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, v)| v.clone())
            };
            let id = field("msgid").filter(|id| !id.is_empty())?;
            Some(Entry {
                key: (field("msgctxt").unwrap_or_default(), id),
                translations: fields
                    .iter()
                    .filter(|(n, _)| n.starts_with("msgstr"))
                    .map(|(_, v)| v.clone())
                    .collect(),
                fuzzy: block
                    .lines()
                    .any(|l| l.starts_with("#,") && l.contains("fuzzy")),
            })
        })
        .collect()
}

fn italian() -> Vec<Entry> {
    let path = manifest().join("lang/it/LC_MESSAGES/diskdome-ui.po");
    po_entries(
        &fs::read_to_string(path)
            .expect("test data is valid")
            .replace("\r\n", "\n"),
    )
}

#[test]
fn every_string_is_in_the_italian_translation_and_nothing_else() {
    let sources = sources();
    let italian: BTreeSet<Key> = italian().into_iter().map(|e| e.key).collect();
    let missing: Vec<_> = sources.difference(&italian).collect();
    let stale: Vec<_> = italian.difference(&sources).collect();
    assert!(
        missing.is_empty(),
        "not in the Italian .po (run ./scripts/translations.ps1, then translate): {missing:?}"
    );
    assert!(
        stale.is_empty(),
        "no longer in the sources, remove them from the Italian .po: {stale:?}"
    );
}

#[test]
fn every_italian_string_is_translated() {
    let untranslated: Vec<_> = italian()
        .into_iter()
        .filter(|e| {
            e.fuzzy || e.translations.is_empty() || e.translations.iter().any(String::is_empty)
        })
        .map(|e| e.key)
        .collect();
    assert!(
        untranslated.is_empty(),
        "translate (and drop any fuzzy mark): {untranslated:?}"
    );
}

#[test]
fn the_reader_finds_contexts_and_continued_strings() {
    let source = "component A { t: @tr(\"One\"); u: @tr(\"Menu\" => \"Open {}\", x); }\nglobal B { v: @tr(\"Two\"); }";
    assert_eq!(
        source_keys(source),
        vec![
            ("A".to_owned(), "One".to_owned()),
            ("Menu".to_owned(), "Open {}".to_owned()),
            ("B".to_owned(), "Two".to_owned()),
        ]
    );
    let po = "msgctxt \"A\"\nmsgid \"\"\n\"Long \"\n\"text\"\nmsgstr \"Testo \"\n\"lungo\"";
    let entries = po_entries(po);
    assert_eq!(entries[0].key, ("A".to_owned(), "Long text".to_owned()));
    assert_eq!(entries[0].translations, vec!["Testo lungo".to_owned()]);
}

#[test]
fn italian_is_bundled_and_nothing_else() {
    let _headless = Headless::install(320, 200).expect("the headless platform installs");
    let _window = MainWindow::new().expect("the window builds");
    assert!(slint::select_bundled_translation("it").is_ok());
    assert!(slint::select_bundled_translation("en").is_ok());
    match slint::select_bundled_translation("fr") {
        Err(SelectBundledTranslationError::LanguageNotFound {
            available_languages,
        }) => {
            let languages: Vec<&str> = available_languages.iter().map(|l| l.as_str()).collect();
            assert_eq!(languages, ["", "it"], "the English source and Italian");
        }
        other => panic!("French is not bundled: {other:?}"),
    }
}
