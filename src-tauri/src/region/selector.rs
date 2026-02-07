use crate::types::WindowInfo;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};

/// Enumerate visible windows using Win32 EnumWindows
pub fn enumerate_visible_windows() -> Result<Vec<WindowInfo>, String> {
    let mut windows: Vec<WindowInfo> = Vec::new();

    unsafe {
        let result = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(enum_callback),
            LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
        );
        if result.is_err() {
            return Err("EnumWindows failed".into());
        }
    }

    Ok(windows)
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Graphics::Dwm::*;

    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    // Skip invisible windows
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    // Skip minimized windows
    if IsIconic(hwnd).as_bool() {
        return BOOL(1);
    }

    // Get window title
    let mut title_buf = [0u16; 256];
    let len = GetWindowTextW(hwnd, &mut title_buf);
    if len == 0 {
        return BOOL(1);
    }
    let title = String::from_utf16_lossy(&title_buf[..len as usize]);

    // Skip empty titles and known system windows
    if title.is_empty() || title == "Program Manager" || title == "Windows Input Experience" {
        return BOOL(1);
    }

    // Check if window is cloaked (hidden by virtual desktop)
    let mut cloaked: i32 = 0;
    let hr = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut _,
        std::mem::size_of::<i32>() as u32,
    );
    if hr.is_ok() && cloaked != 0 {
        return BOOL(1);
    }

    // Get window rect
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return BOOL(1);
    }

    let width = (rect.right - rect.left) as u32;
    let height = (rect.bottom - rect.top) as u32;

    // Skip tiny windows
    if width < 50 || height < 50 {
        return BOOL(1);
    }

    windows.push(WindowInfo {
        title,
        x: rect.left,
        y: rect.top,
        width,
        height,
    });

    BOOL(1)
}
