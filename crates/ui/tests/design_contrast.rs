//! The colour tokens against the contrast rules of `docs/design-system.md` § 7 and § 10 (card 4.T9), in the dark and
//! the light theme, read from the compiled `Theme` and `Data` globals (WCAG 2.1 contrast, OKLab distance):
//! text ≥ 4.5:1 on `bg`, `surface` and `raised`, and on a selected row (`accent-soft` over `surface`, with the ratios
//! the spec quotes); `ink-faint` ≥ 4.0:1; the focus ring and the checkbox edge ≥ 3:1; `on-accent` on `accent`,
//! `on-data` on every category and the age labels ≥ 4.5:1; the chart marks ≥ 3:1 on their grounds; two categories at
//! least ΔE OK 0.09 apart.
//!
//! Blind spot: the Soft and Vivid palettes (§ 3.2) are not computed yet (`Data.palette` shows the standard colours);
//! they join this test with their transform.

#![cfg(feature = "kit")]

use diskdome_ui::generated::{
    AgeBucket, Data, DataShade, FileCategory, KitLabelHarness, Theme, ThemeMode,
};
use diskdome_ui::headless::Headless;
use slint::{Color, ComponentHandle};

/// § 7: text on its ground.
const TEXT: f64 = 4.5;
/// § 7: `ink-faint`, only for disabled text, placeholders and the dash of missing values.
const FAINT: f64 = 4.0;
/// § 7: the focus ring, checkbox and radio boxes; § 4.7: chart marks.
const GRAPHIC: f64 = 3.0;
/// § 10: the smallest ΔE OK between two categories.
const CATEGORY_DISTANCE: f64 = 0.09;

const CATEGORIES: [FileCategory; 12] = [
    FileCategory::Video,
    FileCategory::Image,
    FileCategory::Audio,
    FileCategory::Document,
    FileCategory::Game,
    FileCategory::App,
    FileCategory::Data,
    FileCategory::Code,
    FileCategory::Build,
    FileCategory::Archive,
    FileCategory::System,
    FileCategory::Other,
];
const AGES: [AgeBucket; 6] = [
    AgeBucket::D7,
    AgeBucket::M1,
    AgeBucket::M6,
    AgeBucket::Y1,
    AgeBucket::Y3,
    AgeBucket::Older,
];

/// sRGB channel to linear light (WCAG 2.1).
fn linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.1 relative luminance of an opaque colour.
fn luminance(c: Color) -> f64 {
    0.2126 * linear(c.red()) + 0.7152 * linear(c.green()) + 0.0722 * linear(c.blue())
}

/// WCAG 2.1 contrast ratio between two opaque colours.
fn contrast(a: Color, b: Color) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// A translucent colour laid over an opaque one, as the renderer composites it.
fn over(top: Color, ground: Color) -> Color {
    let alpha = f64::from(top.alpha()) / 255.0;
    let mix = |t: u8, g: u8| (f64::from(t) * alpha + f64::from(g) * (1.0 - alpha)).round() as u8;
    Color::from_rgb_u8(
        mix(top.red(), ground.red()),
        mix(top.green(), ground.green()),
        mix(top.blue(), ground.blue()),
    )
}

/// OKLab coordinates of an opaque colour (Björn Ottosson's matrices).
fn oklab(c: Color) -> [f64; 3] {
    let (r, g, b) = (linear(c.red()), linear(c.green()), linear(c.blue()));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

fn delta_e_ok(a: Color, b: Color) -> f64 {
    let (a, b) = (oklab(a), oklab(b));
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Collects every pair below its threshold, so one run lists them all.
#[derive(Default)]
struct Report(Vec<String>);

impl Report {
    fn at_least(&mut self, what: &str, fg: Color, bg: Color, threshold: f64) {
        let ratio = contrast(fg, bg);
        if ratio < threshold {
            self.0.push(format!("{what}: {ratio:.2}:1 < {threshold}:1"));
        }
    }
}

/// The ratio rounded as the spec writes it, one decimal.
fn one_decimal(ratio: f64) -> f64 {
    (ratio * 10.0).round() / 10.0
}

/// One test: the headless platform is installed once per thread.
#[test]
fn colour_tokens_meet_the_contrast_rules() {
    let _headless = Headless::install(64, 64).expect("headless platform");
    let window = KitLabelHarness::new().expect("the harness compiles");
    let theme = window.global::<Theme>();
    let data = window.global::<Data>();
    let mut report = Report::default();

    // § 7 quotes the selected row's ratios, dark then light: `ink` 9.7 / 11.6, `ink-muted` 5.3 / 5.8.
    for (mode, name, ink_on_selected, muted_on_selected) in [
        (ThemeMode::Dark, "dark", 9.7, 5.3),
        (ThemeMode::Light, "light", 11.6, 5.8),
    ] {
        theme.set_mode(mode);
        let grounds = [
            ("bg", theme.get_bg()),
            ("surface", theme.get_surface()),
            ("raised", theme.get_raised()),
        ];
        let texts = [
            ("ink", theme.get_ink()),
            ("ink-muted", theme.get_ink_muted()),
            ("accent-text", theme.get_accent_text()),
            ("danger", theme.get_danger()),
            ("warning", theme.get_warning()),
            ("success", theme.get_success()),
        ];
        for (ground, g) in grounds {
            for (text, t) in texts {
                report.at_least(&format!("{name}: {text} on {ground}"), t, g, TEXT);
            }
            report.at_least(
                &format!("{name}: ink-faint on {ground}"),
                theme.get_ink_faint(),
                g,
                FAINT,
            );
            report.at_least(
                &format!("{name}: focus on {ground}"),
                theme.get_focus(),
                g,
                GRAPHIC,
            );
            report.at_least(
                &format!("{name}: line-strong on {ground}"),
                theme.get_line_strong(),
                g,
                GRAPHIC,
            );
        }

        // A selected row: `accent-soft` laid over `surface`.
        let selected = over(theme.get_accent_soft(), theme.get_surface());
        for (text, t, quoted) in [
            ("ink", theme.get_ink(), ink_on_selected),
            ("ink-muted", theme.get_ink_muted(), muted_on_selected),
        ] {
            report.at_least(
                &format!("{name}: {text} on a selected row"),
                t,
                selected,
                TEXT,
            );
            let ratio = one_decimal(contrast(t, selected));
            if (ratio - quoted).abs() > f64::EPSILON {
                report.0.push(format!(
                    "{name}: {text} on a selected row is {ratio}:1, § 7 says {quoted}:1"
                ));
            }
        }

        report.at_least(
            &format!("{name}: on-accent on accent"),
            theme.get_on_accent(),
            theme.get_accent(),
            TEXT,
        );

        // Chart marks on the grounds charts are drawn on.
        for (mark, m) in [
            ("chart-space", theme.get_chart_space()),
            ("delta-up", theme.get_delta_up()),
            ("delta-down", theme.get_delta_down()),
        ] {
            for (ground, g) in [("bg", theme.get_bg()), ("surface", theme.get_surface())] {
                report.at_least(&format!("{name}: {mark} on {ground}"), m, g, GRAPHIC);
            }
        }
    }

    // Data colours are the same in both themes (§ 3.2).
    for category in CATEGORIES {
        report.at_least(
            &format!("on-data on {category:?}"),
            data.invoke_on_data(category),
            data.invoke_colour(category, DataShade::Base),
            TEXT,
        );
    }
    for bucket in AGES {
        report.at_least(
            &format!("on-age on {bucket:?}"),
            data.invoke_on_age(bucket),
            data.invoke_age(bucket),
            TEXT,
        );
    }
    for (i, a) in CATEGORIES.iter().enumerate() {
        for b in &CATEGORIES[i + 1..] {
            let distance = delta_e_ok(
                data.invoke_colour(*a, DataShade::Base),
                data.invoke_colour(*b, DataShade::Base),
            );
            if distance < CATEGORY_DISTANCE {
                report.0.push(format!(
                    "{a:?} and {b:?}: ΔE OK {distance:.3} < {CATEGORY_DISTANCE}"
                ));
            }
        }
    }

    assert!(
        report.0.is_empty(),
        "contrast rules broken:\n{}",
        report.0.join("\n")
    );
}
