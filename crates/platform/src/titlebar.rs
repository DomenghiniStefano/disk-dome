//! DiskDome's own title bar on Windows (spike, card 4.T1; `docs/design-system.md` § 5.27, § 12 test 1).
//!
//! The window keeps its native frame styles (caption, thick frame, system menu), so Windows still gives it Aero Snap,
//! Snap Layouts, the Alt+Space menu, the minimise / maximise animations and the resize borders; only the caption is
//! taken away (`WM_NCCALCSIZE`) and the interface draws its own bar in that space. `WM_NCHITTEST` then tells Windows
//! which parts of that bar are the caption (drag, double-click, right-click menu), which are the maximise button
//! (Snap Layouts on Windows 11) and which belong to the interface (tabs, minimise, close).
//!
//! On other systems [`install`] does nothing and returns `false`: the window keeps its native title bar.

use raw_window_handle::RawWindowHandle;

/// The title bar's regions, in logical pixels from the window's top-left corner.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Regions {
    /// Height of the bar.
    pub height: f32,
    /// Horizontal spans inside the bar that the interface handles (tabs, «+», minimise, close).
    pub interactive: Vec<(f32, f32)>,
    /// Horizontal span of the maximise button, handled by Windows so Snap Layouts appears on it.
    pub maximize: (f32, f32),
}

/// What happens to the parts Windows handles, for the interface to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    MaximizeHover(bool),
    MaximizePressed(bool),
    Maximized(bool),
}

/// Takes over the title bar of the window. Returns `false` where there is nothing to do (not Windows, not a
/// Win32 window).
pub fn install(handle: RawWindowHandle, on_event: Box<dyn Fn(Event)>) -> bool {
    imp::install(handle, on_event)
}

/// Sets the regions [`install`] hit-tests against. Call it whenever the layout of the bar changes.
pub fn set_regions(regions: Regions) {
    imp::set_regions(regions);
}

#[cfg(not(windows))]
mod imp {
    use super::{Event, RawWindowHandle, Regions};

    pub fn install(_: RawWindowHandle, _: Box<dyn Fn(Event)>) -> bool {
        false
    }

    pub fn set_regions(_: Regions) {}
}

#[cfg(windows)]
mod imp {
    use std::cell::RefCell;
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::Controls::MARGINS;
    use windows_sys::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClientRect, HTCAPTION, HTCLIENT, HTMAXBUTTON, HTTOP, HTTOPLEFT, HTTOPRIGHT, IsZoomed,
        MINMAXINFO, NCCALCSIZE_PARAMS, SIZE_MAXIMIZED, SM_CXPADDEDBORDER, SM_CYCAPTION,
        SM_CYSIZEFRAME, SW_MAXIMIZE, SW_RESTORE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, ShowWindow, WM_GETMINMAXINFO, WM_MOUSEMOVE,
        WM_NCCALCSIZE, WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDBLCLK, WM_NCLBUTTONDOWN,
        WM_NCLBUTTONUP, WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_SIZE,
    };

    use super::{Event, RawWindowHandle, Regions};

    const SUBCLASS_ID: usize = 0x4444_5442; // "DDTB"

    struct State {
        regions: Regions,
        on_event: Box<dyn Fn(Event)>,
        hovered: bool,
        pressed: bool,
    }

    thread_local! {
        // The window procedure runs on the thread that created the window, which is the one that installs.
        static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
    }

    fn emit(event: Event) {
        STATE.with_borrow(|s| {
            if let Some(s) = s {
                (s.on_event)(event);
            }
        });
    }

    fn set_hover(hovered: bool) {
        let changed = STATE.with_borrow_mut(|s| match s {
            Some(s) if s.hovered != hovered => {
                s.hovered = hovered;
                if !hovered {
                    s.pressed = false;
                }
                true
            }
            Some(_) | None => false,
        });
        if changed {
            emit(Event::MaximizeHover(hovered));
            if !hovered {
                emit(Event::MaximizePressed(false));
            }
        }
    }

    pub fn install(handle: RawWindowHandle, on_event: Box<dyn Fn(Event)>) -> bool {
        let RawWindowHandle::Win32(win32) = handle else {
            return false;
        };
        let hwnd = win32.hwnd.get() as HWND;
        STATE.set(Some(State {
            regions: Regions::default(),
            on_event,
            hovered: false,
            pressed: false,
        }));
        // SAFETY: `hwnd` is the live window Slint just handed us, used on its own thread.
        unsafe {
            if SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0) == 0 {
                STATE.set(None);
                return false;
            }
            // One pixel of the DWM frame under the client area keeps the shadow and the top edge on Windows 10.
            let margins = MARGINS {
                cxLeftWidth: 0,
                cxRightWidth: 0,
                cyTopHeight: 1,
                cyBottomHeight: 0,
            };
            DwmExtendFrameIntoClientArea(hwnd, &margins);
            // Recompute the frame now, without the caption.
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            emit(Event::Maximized(IsZoomed(hwnd) != 0));
        }
        true
    }

    pub fn set_regions(regions: Regions) {
        STATE.with_borrow_mut(|s| {
            if let Some(s) = s {
                s.regions = regions;
            }
        });
    }

    /// The height of the resize border at the window's DPI, in physical pixels.
    fn frame_thickness(hwnd: HWND) -> i32 {
        // SAFETY: plain queries on a live window.
        unsafe {
            let dpi = GetDpiForWindow(hwnd);
            GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi)
                + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi)
        }
    }

    fn hit_test(hwnd: HWND, lparam: LPARAM) -> Option<u32> {
        let mut point = POINT {
            x: i32::from((lparam & 0xffff) as u16 as i16),
            y: i32::from(((lparam >> 16) & 0xffff) as u16 as i16),
        };
        let mut client = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: plain queries on a live window, into local values.
        let (dpi, zoomed) = unsafe {
            ScreenToClient(hwnd, &mut point);
            GetClientRect(hwnd, &mut client);
            (GetDpiForWindow(hwnd), IsZoomed(hwnd) != 0)
        };
        let scale = dpi as f32 / 96.0;
        let frame = frame_thickness(hwnd);
        if !zoomed && point.y < frame {
            return Some(if point.x < frame * 2 {
                HTTOPLEFT
            } else if point.x >= client.right - frame * 2 {
                HTTOPRIGHT
            } else {
                HTTOP
            });
        }
        STATE.with_borrow(|s| {
            let r = &s.as_ref()?.regions;
            let (x, y) = (point.x as f32 / scale, point.y as f32 / scale);
            if y >= r.height {
                return None;
            }
            if x >= r.maximize.0 && x < r.maximize.1 {
                return Some(HTMAXBUTTON);
            }
            if r.interactive.iter().any(|&(a, b)| x >= a && x < b) {
                return None;
            }
            Some(HTCAPTION)
        })
    }

    fn is_max(wparam: WPARAM) -> bool {
        wparam == HTMAXBUTTON as WPARAM
    }

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        _data: usize,
    ) -> LRESULT {
        // SAFETY: called by Windows for our live window with the arguments of the message.
        unsafe {
            match msg {
                WM_NCCALCSIZE if wparam != 0 => {
                    let params = lparam as *mut NCCALCSIZE_PARAMS;
                    let top = (*params).rgrc[0].top;
                    let result = DefSubclassProc(hwnd, msg, wparam, lparam);
                    if result != 0 {
                        return result;
                    }
                    // Keep the side and bottom borders Windows computed; take the caption away. Maximised, the
                    // window overhangs the screen by the frame, so the client starts one frame lower.
                    (*params).rgrc[0].top = top
                        + if IsZoomed(hwnd) != 0 {
                            frame_thickness(hwnd)
                        } else {
                            0
                        };
                    0
                }
                WM_GETMINMAXINFO => {
                    let result = DefSubclassProc(hwnd, msg, wparam, lparam);
                    // winit sizes the minimum as if the caption were there; it is not, so the client
                    // would be a caption taller than the minimum the interface asked for.
                    let info = lparam as *mut MINMAXINFO;
                    let caption = GetSystemMetricsForDpi(SM_CYCAPTION, GetDpiForWindow(hwnd));
                    if (*info).ptMinTrackSize.y > caption {
                        (*info).ptMinTrackSize.y -= caption;
                    }
                    result
                }
                WM_NCHITTEST => {
                    let native = DefSubclassProc(hwnd, msg, wparam, lparam);
                    if native != HTCLIENT as LRESULT {
                        return native;
                    }
                    hit_test(hwnd, lparam).map_or(native, |hit| hit as LRESULT)
                }
                WM_NCMOUSEMOVE => {
                    // No TrackMouseEvent here: it would replace the hover tracking Windows 11 uses to open
                    // Snap Layouts. Leaving the button is seen by the next mouse message on another part.
                    set_hover(is_max(wparam));
                    DefSubclassProc(hwnd, msg, wparam, lparam)
                }
                WM_NCMOUSELEAVE | WM_MOUSEMOVE => {
                    set_hover(false);
                    DefSubclassProc(hwnd, msg, wparam, lparam)
                }
                WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK if is_max(wparam) => {
                    STATE.with_borrow_mut(|s| {
                        if let Some(s) = s {
                            s.pressed = true;
                        }
                    });
                    emit(Event::MaximizePressed(true));
                    0
                }
                WM_NCLBUTTONUP if is_max(wparam) => {
                    let was_pressed = STATE.with_borrow_mut(|s| {
                        s.as_mut()
                            .is_some_and(|s| std::mem::replace(&mut s.pressed, false))
                    });
                    emit(Event::MaximizePressed(false));
                    if was_pressed {
                        ShowWindow(
                            hwnd,
                            if IsZoomed(hwnd) != 0 {
                                SW_RESTORE
                            } else {
                                SW_MAXIMIZE
                            },
                        );
                    }
                    0
                }
                WM_SIZE => {
                    emit(Event::Maximized(wparam == SIZE_MAXIMIZED as WPARAM));
                    DefSubclassProc(hwnd, msg, wparam, lparam)
                }
                WM_NCDESTROY => {
                    RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID);
                    STATE.set(None);
                    DefSubclassProc(hwnd, msg, wparam, lparam)
                }
                _ => DefSubclassProc(hwnd, msg, wparam, lparam),
            }
        }
    }
}
