//! Windows' own tools for the space Windows keeps for itself (`docs/design-system.md` § 4.5 «Windows' space», § 12
//! test 11). DiskDome only opens them, never elevated: a tool that needs administrator rights shows its own prompt
//! (rule 4), and DiskDome never touches the files they manage (rule 3).

/// A tool of «Windows' space», one per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsTool {
    /// The page file: Performance Options (its Advanced tab holds virtual memory).
    PerformanceOptions,
    /// The hibernation file: Power Options.
    PowerOptions,
    /// Restore points (`System Volume Information`): System Protection.
    SystemProtection,
    /// Updates, components and temporary files: Disk Cleanup.
    DiskCleanup,
    /// The Recycle Bin, in File Explorer.
    RecycleBin,
}

impl WindowsTool {
    pub const ALL: [WindowsTool; 5] = [
        WindowsTool::PerformanceOptions,
        WindowsTool::PowerOptions,
        WindowsTool::SystemProtection,
        WindowsTool::DiskCleanup,
        WindowsTool::RecycleBin,
    ];

    /// The program and its arguments, as Windows 10 and 11 name them.
    pub fn command(self) -> (&'static str, &'static str) {
        match self {
            WindowsTool::PerformanceOptions => ("SystemPropertiesPerformance.exe", ""),
            WindowsTool::PowerOptions => ("control.exe", "/name Microsoft.PowerOptions"),
            WindowsTool::SystemProtection => ("SystemPropertiesProtection.exe", ""),
            WindowsTool::DiskCleanup => ("cleanmgr.exe", ""),
            WindowsTool::RecycleBin => ("explorer.exe", "shell:RecycleBinFolder"),
        }
    }
}

/// Opens a tool with the user's own rights, through the shell (so a tool that needs administrator rights shows its
/// own UAC prompt instead of failing). `false` when Windows refuses or on another system.
#[cfg(windows)]
pub fn open(tool: WindowsTool) -> bool {
    use std::ptr::null;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    /// `ShellExecuteW` returns a value above this on success.
    const SHELL_OK_ABOVE: isize = 32;

    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };
    let (file, params) = tool.command();
    let (verb, file, params) = (wide("open"), wide(file), wide(params));
    // SAFETY: every string is NUL-terminated and outlives the call; no owner window and no directory are passed.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            null(),
            SW_SHOWNORMAL,
        )
    };
    result as isize > SHELL_OK_ABOVE
}

/// Opens a tool: there are none on this system.
#[cfg(not(windows))]
pub fn open(_tool: WindowsTool) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::WindowsTool;

    #[test]
    fn every_tool_names_a_program_and_never_runas() {
        for tool in WindowsTool::ALL {
            let (file, params) = tool.command();
            assert!(file.ends_with(".exe"), "{tool:?}: a program, {file}");
            assert!(
                !params.contains("runas") && !file.contains("runas"),
                "{tool:?}: DiskDome never elevates (rule 4)"
            );
        }
    }

    /// § 4.5: the five tools are distinct.
    #[test]
    fn the_tools_are_distinct() {
        for (i, a) in WindowsTool::ALL.iter().enumerate() {
            for b in &WindowsTool::ALL[i + 1..] {
                assert_ne!(a.command(), b.command(), "{a:?} and {b:?}");
            }
        }
    }
}
