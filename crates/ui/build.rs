//! Compiles the interface. With a development feature (`kit`, `spikes`) the entry also exports the Kit window
//! (§ 9) and the throwaway proofs, which never reach a release build.

/// The `.slint` entry for the build: the app, or the app plus the development windows.
fn entry() -> &'static str {
    let dev = ["CARGO_FEATURE_KIT", "CARGO_FEATURE_SPIKES"]
        .iter()
        .any(|feature| std::env::var_os(feature).is_some());
    if dev {
        "ui/dev-app.slint"
    } else {
        "ui/app.slint"
    }
}

/// The `.po` files, one folder per language (`lang/<lang>/LC_MESSAGES/diskdome-ui.po`), built into the binary.
/// Slint picks the one matching the OS locale and falls back to the English source.
const TRANSLATIONS: &str = "lang";

/// The design system's Slint library, imported as `@dome-ui`: its project file names `index.slint` as the only file
/// an app may import (`docs/design-system.md` § 10, rule `boundary`).
const DOME_UI: &str = "../dome-ui/ui/slint-project.json";

fn main() {
    let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOME_UI);
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations(TRANSLATIONS)
        .with_library_paths(std::collections::HashMap::from([(
            "dome-ui".to_owned(),
            library,
        )]));
    slint_build::compile_with_config(entry(), config).expect("the Slint interface must compile");
}
