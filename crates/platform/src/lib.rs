//! Operating-system services outside the scanners. For now: the process's memory, which every
//! benchmark reports. The shell actions (Recycle Bin, Explorer, Properties) will live here too, so
//! no other crate needs `#[cfg(windows)]`.

/// Memory of the current process, in bytes.
#[derive(Debug, Clone, Copy, Default)]
pub struct Memory {
    /// Physical memory in use now (working set).
    pub working_set: u64,
    pub peak_working_set: u64,
    /// Private committed memory now (what Task Manager calls "commit size").
    pub private: u64,
    pub peak_private: u64,
}

/// The current process's memory, or `None` where it is not implemented yet.
#[cfg(windows)]
pub fn memory() -> Option<Memory> {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    // SAFETY: plain-old-data structure, zero is a valid value.
    let mut c: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
    c.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    // SAFETY: `c` is a PROCESS_MEMORY_COUNTERS_EX of the size passed; the pseudo-handle of the
    // current process needs no closing.
    let ok = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&raw mut c).cast::<PROCESS_MEMORY_COUNTERS>(),
            c.cb,
        )
    };
    (ok != 0).then_some(Memory {
        working_set: c.WorkingSetSize as u64,
        peak_working_set: c.PeakWorkingSetSize as u64,
        private: c.PrivateUsage as u64,
        peak_private: c.PeakPagefileUsage as u64,
    })
}

/// The current process's memory, or `None` where it is not implemented yet.
#[cfg(not(windows))]
pub fn memory() -> Option<Memory> {
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn memory_is_reported_on_windows() {
        let m = super::memory();
        if cfg!(windows) {
            let m = m.expect("available on Windows");
            assert!(m.working_set > 0 && m.peak_working_set >= m.working_set);
        }
    }
}
