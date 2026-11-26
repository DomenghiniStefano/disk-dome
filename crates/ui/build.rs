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

fn main() {
    slint_build::compile(entry()).expect("the Slint interface must compile");
}
