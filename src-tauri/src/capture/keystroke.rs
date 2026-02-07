use crate::types::KeystrokeEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct KeystrokeCaptureHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub events: Arc<Mutex<Vec<KeystrokeEvent>>>,
    pub join_handle: Option<std::thread::JoinHandle<()>>,
}

// Virtual key codes for modifiers
const VK_SHIFT: u32 = 0x10;
const VK_CTRL: u32 = 0x11;
const VK_ALT: u32 = 0x12;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;

// Mouse buttons
const VK_LBUTTON: u32 = 0x01;
const VK_RBUTTON: u32 = 0x02;
const VK_MBUTTON: u32 = 0x04;

fn is_modifier(vk: u32) -> bool {
    matches!(vk, VK_SHIFT | VK_CTRL | VK_ALT | VK_LWIN | VK_RWIN
        | 0xA0 | 0xA1  // LShift, RShift
        | 0xA2 | 0xA3  // LCtrl, RCtrl
        | 0xA4 | 0xA5  // LAlt, RAlt
    )
}

/// Build modifier prefix from current key states
fn modifier_prefix(get_state: impl Fn(u32) -> bool) -> String {
    let mut parts = Vec::new();
    if get_state(VK_CTRL) { parts.push("Ctrl"); }
    if get_state(VK_SHIFT) { parts.push("Shift"); }
    if get_state(VK_ALT) { parts.push("Alt"); }
    if get_state(VK_LWIN) || get_state(VK_RWIN) { parts.push("Win"); }
    if parts.is_empty() {
        String::new()
    } else {
        let mut s = parts.join("+");
        s.push('+');
        s
    }
}

/// Start capturing keystrokes and mouse clicks
pub fn start_capture(start_time: Instant) -> Result<KeystrokeCaptureHandle, String> {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));

    let stop = stop_flag.clone();
    let evts = events.clone();

    let handle = std::thread::spawn(move || {
        use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

        let mut prev_states = [false; 256];

        let get_pressed = |vk: u32| -> bool {
            let state = unsafe { GetAsyncKeyState(vk as i32) };
            (state & (0x8000u16 as i16)) != 0
        };

        while !stop.load(Ordering::Relaxed) {
            for vk in 0u32..256 {
                let pressed = get_pressed(vk);
                let was_pressed = prev_states[vk as usize];

                if pressed && !was_pressed {
                    // Skip modifier-only presses — they'll be part of combos
                    if is_modifier(vk) {
                        prev_states[vk as usize] = pressed;
                        continue;
                    }

                    let base_name = vk_to_name(vk);
                    if base_name.is_empty() {
                        prev_states[vk as usize] = pressed;
                        continue;
                    }

                    let prefix = modifier_prefix(&get_pressed);
                    let key_name = format!("{}{}", prefix, base_name);

                    let timestamp_ms = start_time.elapsed().as_millis() as u64;
                    if let Ok(mut e) = evts.lock() {
                        e.push(KeystrokeEvent { timestamp_ms, key_name });
                    }
                }
                prev_states[vk as usize] = pressed;
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    Ok(KeystrokeCaptureHandle {
        stop_flag,
        events,
        join_handle: Some(handle),
    })
}

/// Stop capturing and return collected events
pub fn stop_capture(handle: &mut KeystrokeCaptureHandle) -> Vec<KeystrokeEvent> {
    handle.stop_flag.store(true, Ordering::Relaxed);
    if let Some(h) = handle.join_handle.take() {
        let _ = h.join();
    }
    handle.events.lock().map(|e| e.clone()).unwrap_or_default()
}

/// Convert virtual key code to human-readable name
fn vk_to_name(vk: u32) -> String {
    match vk {
        // Mouse
        VK_LBUTTON => "Clic".into(),
        VK_RBUTTON => "Clic Droit".into(),
        VK_MBUTTON => "Clic Milieu".into(),
        // Navigation
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x14 => "CapsLock".into(),
        0x1B => "Esc".into(),
        0x20 => "Space".into(),
        0x21 => "PgUp".into(),
        0x22 => "PgDn".into(),
        0x23 => "End".into(),
        0x24 => "Home".into(),
        0x25 => "\u{2190}".into(),  // ←
        0x26 => "\u{2191}".into(),  // ↑
        0x27 => "\u{2192}".into(),  // →
        0x28 => "\u{2193}".into(),  // ↓
        0x2D => "Insert".into(),
        0x2E => "Delete".into(),
        // Numbers
        0x30..=0x39 => format!("{}", vk - 0x30),
        // Letters
        0x41..=0x5A => format!("{}", (vk as u8 as char)),
        // Function keys
        0x70..=0x87 => format!("F{}", vk - 0x6F), // F1–F24
        // Numpad
        0x60..=0x69 => format!("Num{}", vk - 0x60),
        0x6A => "*".into(),
        0x6B => "+".into(),
        0x6D => "-".into(),
        0x6E => ".".into(),
        0x6F => "/".into(),
        // Punctuation
        0xBA => ";".into(),
        0xBB => "=".into(),
        0xBC => ",".into(),
        0xBD => "-".into(),
        0xBE => ".".into(),
        0xBF => "/".into(),
        0xC0 => "`".into(),
        0xDB => "[".into(),
        0xDC => "\\".into(),
        0xDD => "]".into(),
        0xDE => "'".into(),
        // PrintScreen, ScrollLock, Pause
        0x2C => "PrtSc".into(),
        0x91 => "ScrollLock".into(),
        0x13 => "Pause".into(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── vk_to_name ──

    #[test]
    fn test_vk_letters() {
        assert_eq!(vk_to_name(0x41), "A");
        assert_eq!(vk_to_name(0x5A), "Z");
    }

    #[test]
    fn test_vk_digits() {
        assert_eq!(vk_to_name(0x30), "0");
        assert_eq!(vk_to_name(0x39), "9");
    }

    #[test]
    fn test_vk_function_keys() {
        assert_eq!(vk_to_name(0x70), "F1");
        assert_eq!(vk_to_name(0x7B), "F12");
        assert_eq!(vk_to_name(0x87), "F24");
    }

    #[test]
    fn test_vk_arrows() {
        assert_eq!(vk_to_name(0x25), "\u{2190}"); // ←
        assert_eq!(vk_to_name(0x26), "\u{2191}"); // ↑
        assert_eq!(vk_to_name(0x27), "\u{2192}"); // →
        assert_eq!(vk_to_name(0x28), "\u{2193}"); // ↓
    }

    #[test]
    fn test_vk_mouse() {
        assert_eq!(vk_to_name(VK_LBUTTON), "Clic");
        assert_eq!(vk_to_name(VK_RBUTTON), "Clic Droit");
        assert_eq!(vk_to_name(VK_MBUTTON), "Clic Milieu");
    }

    #[test]
    fn test_vk_special_keys() {
        assert_eq!(vk_to_name(0x0D), "Enter");
        assert_eq!(vk_to_name(0x20), "Space");
        assert_eq!(vk_to_name(0x08), "Backspace");
        assert_eq!(vk_to_name(0x09), "Tab");
        assert_eq!(vk_to_name(0x1B), "Esc");
    }

    #[test]
    fn test_vk_numpad() {
        assert_eq!(vk_to_name(0x60), "Num0");
        assert_eq!(vk_to_name(0x69), "Num9");
        assert_eq!(vk_to_name(0x6A), "*");
        assert_eq!(vk_to_name(0x6B), "+");
    }

    #[test]
    fn test_vk_unknown() {
        assert_eq!(vk_to_name(0xFF), "");
    }

    // ── is_modifier ──

    #[test]
    fn test_is_modifier_true() {
        assert!(is_modifier(VK_SHIFT));
        assert!(is_modifier(VK_CTRL));
        assert!(is_modifier(VK_ALT));
        assert!(is_modifier(VK_LWIN));
        assert!(is_modifier(0xA0)); // LShift
        assert!(is_modifier(0xA3)); // RCtrl
    }

    #[test]
    fn test_is_modifier_false() {
        assert!(!is_modifier(0x41)); // A
        assert!(!is_modifier(0x0D)); // Enter
        assert!(!is_modifier(0x70)); // F1
        assert!(!is_modifier(VK_LBUTTON));
    }

    // ── modifier_prefix ──

    #[test]
    fn test_prefix_none() {
        assert_eq!(modifier_prefix(|_| false), "");
    }

    #[test]
    fn test_prefix_ctrl() {
        assert_eq!(modifier_prefix(|vk| vk == VK_CTRL), "Ctrl+");
    }

    #[test]
    fn test_prefix_ctrl_shift() {
        assert_eq!(
            modifier_prefix(|vk| vk == VK_CTRL || vk == VK_SHIFT),
            "Ctrl+Shift+"
        );
    }

    #[test]
    fn test_prefix_all() {
        let result = modifier_prefix(|vk| {
            matches!(vk, VK_CTRL | VK_SHIFT | VK_ALT | VK_LWIN)
        });
        assert_eq!(result, "Ctrl+Shift+Alt+Win+");
    }
}
