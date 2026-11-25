//! Operating-system services outside the scanners. For now: the process's memory, which every
//! benchmark reports, and the window's own title bar ([`titlebar`]). The shell actions (Recycle Bin,
//! Explorer, Properties) will live here too, so no other crate needs `#[cfg(windows)]`.

pub mod titlebar;

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

/// What Windows reports for one file: the values File Explorer's Properties shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileInfo {
    /// «Size».
    pub size: u64,
    /// «Size on disk» as the file system reports it (`AllocationSize`).
    pub alloc: u64,
    /// `GetCompressedFileSizeW`: the bytes really stored for compressed and sparse files.
    pub compressed: u64,
    /// Last modification, Unix seconds.
    pub mtime: u32,
    /// `FILE_ATTRIBUTE_*`.
    pub attributes: u32,
    /// Number of hard links.
    pub links: u32,
}

/// Reads a file's metadata with the user's own rights. Opens it for attributes only: nothing is read
/// from the file, nothing changes on disk.
#[cfg(windows)]
pub fn file_info(path: &str) -> Option<FileInfo> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_BASIC_INFO, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_STANDARD_INFO, FileBasicInfo, FileStandardInfo, GetCompressedFileSizeW,
        GetFileInformationByHandleEx, OPEN_EXISTING,
    };

    // The long-path prefix `\\?\`, so paths over 260 characters open too.
    const LONG: &str = concat!(r"\", r"\?\");
    let long = if path.starts_with(LONG) {
        path.to_string()
    } else {
        format!("{LONG}{path}")
    };
    let wide: Vec<u16> = long.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is NUL-terminated and outlives the call.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return None;
    }
    // SAFETY: plain-old-data structures, zero is a valid value.
    let mut standard: FILE_STANDARD_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: as above.
    let mut basic: FILE_BASIC_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: the buffers are the structures the classes ask for, with their sizes.
    let ok = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileStandardInfo,
            (&raw mut standard).cast(),
            size_of::<FILE_STANDARD_INFO>() as u32,
        ) != 0
            && GetFileInformationByHandleEx(
                handle,
                FileBasicInfo,
                (&raw mut basic).cast(),
                size_of::<FILE_BASIC_INFO>() as u32,
            ) != 0
    };
    // SAFETY: the handle came from CreateFileW and is closed once.
    unsafe { CloseHandle(handle) };
    if !ok {
        return None;
    }
    let mut high = 0u32;
    // SAFETY: `wide` is NUL-terminated; `high` receives the upper 32 bits.
    let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };
    let compressed = (u64::from(high) << 32) | u64::from(low);
    let ft = u64::try_from(basic.LastWriteTime).unwrap_or(0);
    let mtime = (ft / 10_000_000).saturating_sub(11_644_473_600);
    Some(FileInfo {
        size: u64::try_from(standard.EndOfFile).unwrap_or(0),
        alloc: u64::try_from(standard.AllocationSize).unwrap_or(0),
        compressed,
        mtime: u32::try_from(mtime).unwrap_or(u32::MAX),
        attributes: basic.FileAttributes,
        links: standard.NumberOfLinks,
    })
}

/// Reads a file's metadata: not implemented on this platform yet.
#[cfg(not(windows))]
pub fn file_info(_path: &str) -> Option<FileInfo> {
    None
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
