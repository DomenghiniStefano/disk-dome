//! The Kit window (`docs/design-system.md` § 9): `cargo run -p diskdome-ui --features kit -- --kit` shows every
//! token and every component of the design system in every variant, size and state, and the review board. Only
//! with the `kit` feature, never in a release.
//!
//! `--renderer femtovg|skia|software` picks the renderer for test 7 (§ 12, decision 9); the font is switched in the
//! window itself.

use slint::{BackendSelector, ComponentHandle, Model, SharedString};

use crate::generated::{KitIssueStatus, KitReview, KitRun, KitWindow};

/// The flag that opens the Kit instead of the app.
pub const FLAG: &str = "--kit";

/// The flag followed by the renderer's name.
pub const RENDERER_FLAG: &str = "--renderer";

/// The renderers compared by test 7. All three are compiled in with the `kit` feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    /// What the app uses today: Slint's OpenGL renderer.
    Femtovg,
    Skia,
    /// Slint's CPU renderer.
    Software,
}

impl Renderer {
    /// Slint's name for it, as `SLINT_BACKEND` takes it.
    pub fn name(self) -> &'static str {
        match self {
            Renderer::Femtovg => "femtovg",
            Renderer::Skia => "skia",
            Renderer::Software => "software",
        }
    }

    fn from_name(name: &str) -> Option<Renderer> {
        [Renderer::Femtovg, Renderer::Skia, Renderer::Software]
            .into_iter()
            .find(|renderer| renderer.name() == name)
    }

    /// The renderer named after `--renderer`, femtovg when the flag is absent: with Skia compiled in, Slint
    /// would otherwise default to Skia and the Kit would not show what the app shows. `None` for an unknown name.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Renderer> {
        let mut args = args.into_iter();
        if !args.any(|arg| arg == RENDERER_FLAG) {
            return Some(Renderer::Femtovg);
        }
        args.next().as_deref().and_then(Renderer::from_name)
    }
}

pub fn run() -> Result<(), slint::PlatformError> {
    let Some(renderer) = Renderer::from_args(std::env::args()) else {
        return Err(slint::PlatformError::Other(format!(
            "{RENDERER_FLAG} takes femtovg, skia or software"
        )));
    };
    BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name(renderer.name().into())
        .select()?;
    let kit = open()?;
    kit.global::<KitRun>()
        .set_renderer(SharedString::from(renderer.name()));
    kit.run()
}

/// The Kit with its review board counted, not yet shown.
pub fn open() -> Result<KitWindow, slint::PlatformError> {
    let kit = KitWindow::new()?;
    count_review(&kit);
    Ok(kit)
}

/// Slint cannot count over an array: the top bar's «open» and «proposed» numbers come from here.
fn count_review(kit: &KitWindow) {
    let review = kit.global::<KitReview>();
    let issues = review.get_issues();
    let count =
        |status: KitIssueStatus| issues.iter().filter(|issue| issue.status == status).count();
    review.set_open_count(to_int(count(KitIssueStatus::Open)));
    review.set_proposed_count(to_int(count(KitIssueStatus::Proposed)));
}

/// A handful of issues always fits an `i32`; saturate rather than panic if it ever did not.
fn to_int(count: usize) -> i32 {
    i32::try_from(count).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{RENDERER_FLAG, Renderer};

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_owned()).collect()
    }

    #[test]
    fn the_kit_starts_on_the_apps_renderer() {
        assert_eq!(
            Renderer::from_args(args(&["diskdome", "--kit"])),
            Some(Renderer::Femtovg)
        );
    }

    #[test]
    fn the_renderer_flag_picks_each_renderer() {
        for renderer in [Renderer::Femtovg, Renderer::Skia, Renderer::Software] {
            let picked =
                Renderer::from_args(args(&["diskdome", "--kit", RENDERER_FLAG, renderer.name()]));
            assert_eq!(picked, Some(renderer));
        }
    }

    #[test]
    fn an_unknown_or_missing_renderer_is_refused() {
        assert_eq!(
            Renderer::from_args(args(&["diskdome", RENDERER_FLAG, "vello"])),
            None
        );
        assert_eq!(
            Renderer::from_args(args(&["diskdome", RENDERER_FLAG])),
            None
        );
    }
}
