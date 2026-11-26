//! The icon set (`docs/design-system.md` § 3.12): every SVG in `crates/dome-ui/ui/icons/` has its property in
//! `global Icons` and is shown in the Kit, and the pinned Lucide version is the one `THIRD_PARTY.md` lists.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

fn icons_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../dome-ui/ui/icons")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("test data is valid")
}

fn svg_names() -> BTreeSet<String> {
    fs::read_dir(icons_dir())
        .expect("test data is valid")
        .map(|entry| entry.expect("test data is valid").path())
        .filter(|path| path.extension().is_some_and(|e| e == "svg"))
        .map(|path| {
            path.file_stem()
                .expect("test data is valid")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// `out property <image> folder: @image-url("folder.svg");` → `folder`, checked against its file name.
fn icon_properties() -> BTreeSet<String> {
    let source = read(&icons_dir().join("icons.slint"));
    let property = Regex::new(r#"out property <image> ([\w-]+): @image-url\("([\w-]+)\.svg"\);"#)
        .expect("test data is valid");
    property
        .captures_iter(&source)
        .map(|c| {
            assert_eq!(&c[1], &c[2], "an icon is named as its file");
            c[1].to_owned()
        })
        .collect()
}

fn lucide_version(text: &str) -> Option<String> {
    // `lucide-static 1.71.0` in our files, `lucide-static v1.71.0` in the SVGs' own header.
    Regex::new(r"lucide-static v?(\d+\.\d+\.\d+)")
        .expect("test data is valid")
        .captures(text)
        .map(|c| c[1].to_owned())
}

#[test]
fn every_svg_has_its_property_and_every_property_its_svg() {
    assert_eq!(svg_names(), icon_properties());
}

#[test]
fn the_kit_shows_every_icon() {
    let kit = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/kit/kit.slint"));
    let missing: Vec<String> = icon_properties()
        .into_iter()
        .filter(|name| !kit.contains(&format!("Icons.{name} ")))
        .collect();
    assert!(missing.is_empty(), "missing from the Kit: {missing:?}");
}

#[test]
fn the_pinned_lucide_version_is_the_listed_one() {
    let pinned = lucide_version(&read(&icons_dir().join("icons.slint")));
    let listed = lucide_version(&read(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../THIRD_PARTY.md"),
    ));
    assert!(
        pinned.is_some(),
        "icons.slint names its lucide-static version"
    );
    assert_eq!(pinned, listed);
    for name in svg_names() {
        let svg = read(&icons_dir().join(format!("{name}.svg")));
        assert_eq!(
            lucide_version(&svg),
            pinned,
            "{name}.svg comes from another version"
        );
    }
}
