//! Compiles the interface. With the `kit` feature the entry also exports the Kit window (§ 9), which never
//! reaches a release build.

/// The `.slint` entry for the build: the app, or the app plus the Kit.
fn entry() -> &'static str {
    if std::env::var_os("CARGO_FEATURE_KIT").is_some() {
        "ui/kit-app.slint"
    } else {
        "ui/app.slint"
    }
}

fn main() {
    slint_build::compile(entry()).expect("the Slint interface must compile");
}
