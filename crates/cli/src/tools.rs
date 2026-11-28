//! `windows-tool NAME`: opens one of Windows' own tools of «Windows' space» (`docs/design-system.md` § 4.5), to try
//! each button by hand (§ 12 test 11) before the Overview card exists.

use diskdome_platform::windows_tools::{self, WindowsTool};

/// The names the command takes, one per tool.
const NAMES: [(&str, WindowsTool); 5] = [
    ("performance", WindowsTool::PerformanceOptions),
    ("power", WindowsTool::PowerOptions),
    ("protection", WindowsTool::SystemProtection),
    ("cleanup", WindowsTool::DiskCleanup),
    ("recycle-bin", WindowsTool::RecycleBin),
];

fn parse(name: &str) -> Option<WindowsTool> {
    NAMES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, tool)| *tool)
}

fn names() -> String {
    NAMES.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
}

pub fn run(name: Option<&str>) -> Result<(), String> {
    let name = name.ok_or_else(|| format!("which tool? one of {}", names()))?;
    let tool = parse(name).ok_or_else(|| format!("no tool {name}; one of {}", names()))?;
    if windows_tools::open(tool) {
        println!("opened {tool:?}");
        Ok(())
    } else {
        Err(format!("Windows did not open {tool:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{NAMES, WindowsTool, parse};

    #[test]
    fn every_tool_has_a_name() {
        for tool in WindowsTool::ALL {
            assert!(
                NAMES.iter().any(|(_, t)| *t == tool),
                "{tool:?} has no name"
            );
        }
        assert_eq!(parse("cleanup"), Some(WindowsTool::DiskCleanup));
        assert_eq!(parse("nothing"), None);
    }
}
