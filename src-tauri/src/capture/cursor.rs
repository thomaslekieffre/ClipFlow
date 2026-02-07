use crate::types::{CursorPosition, Region};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct CursorTrackingHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub positions: Arc<Mutex<Vec<CursorPosition>>>,
    pub join_handle: Option<std::thread::JoinHandle<()>>,
}

/// Start tracking cursor position relative to the capture region
pub fn start_tracking(region: &Region, start_time: Instant) -> CursorTrackingHandle {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let positions = Arc::new(Mutex::new(Vec::new()));

    let stop = stop_flag.clone();
    let pos = positions.clone();
    let reg_x = region.x;
    let reg_y = region.y;
    let reg_w = region.width as f64;
    let reg_h = region.height as f64;

    let handle = std::thread::spawn(move || {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

        while !stop.load(Ordering::Relaxed) {
            let mut point = POINT::default();
            let ok = unsafe { GetCursorPos(&mut point) };
            if ok.is_ok() {
                let timestamp_ms = start_time.elapsed().as_millis() as u64;
                let x = ((point.x - reg_x) as f64 / reg_w).clamp(0.0, 1.0);
                let y = ((point.y - reg_y) as f64 / reg_h).clamp(0.0, 1.0);

                if let Ok(mut p) = pos.lock() {
                    p.push(CursorPosition { timestamp_ms, x, y });
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    CursorTrackingHandle {
        stop_flag,
        positions,
        join_handle: Some(handle),
    }
}

/// Stop tracking and return collected positions
pub fn stop_tracking(handle: &mut CursorTrackingHandle) -> Vec<CursorPosition> {
    handle.stop_flag.store(true, Ordering::Relaxed);
    if let Some(h) = handle.join_handle.take() {
        let _ = h.join();
    }
    handle.positions.lock().map(|p| p.clone()).unwrap_or_default()
}
