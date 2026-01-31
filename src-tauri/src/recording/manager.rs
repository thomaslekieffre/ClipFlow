use crate::capture::screen;
use crate::state::AppState;
use crate::types::{Clip, RecordingState, Region, Transition};
use std::sync::Mutex;
use std::time::Instant;

const FRAMERATE: u32 = 30;

pub fn start(state: &Mutex<AppState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if s.recording_state != RecordingState::Idle {
        return Err("Already recording".into());
    }

    // Ensure temp dir exists
    std::fs::create_dir_all(&s.temp_dir).map_err(|e| e.to_string())?;

    let clip_id = uuid::Uuid::new_v4().to_string();
    let clip_path = s.temp_dir.join(format!("{}.mp4", clip_id));

    let child = if let Some(ref region) = s.current_region {
        screen::start_capture(region, &clip_path, FRAMERATE)
    } else {
        screen::start_fullscreen_capture(&clip_path, FRAMERATE)
    }
    .map_err(|e| format!("Failed to start capture: {}", e))?;

    s.ffmpeg_process = Some(child);
    s.recording_start = Some(Instant::now());
    s.current_clip_path = Some(clip_path);
    s.recording_state = RecordingState::Recording;

    Ok(())
}

pub async fn stop(state: &Mutex<AppState>) -> Result<Clip, String> {
    let (mut child, start_time, clip_path, region, has_audio) = {
        let mut s = state.lock().map_err(|e| e.to_string())?;

        if s.recording_state != RecordingState::Recording {
            return Err("Not recording".into());
        }

        // .take() ensures only the first caller gets the process — prevents double-stop
        let child = s.ffmpeg_process.take()
            .ok_or("No FFmpeg process (already stopped?)")?;
        let start_time = s.recording_start.take()
            .ok_or("No recording start time")?;
        let clip_path = s.current_clip_path.take()
            .ok_or("No clip path")?;

        // Mark idle immediately to prevent concurrent stop attempts
        s.recording_state = RecordingState::Idle;
        let region = s.current_region.clone().unwrap_or(Region {
            x: 0, y: 0, width: 1920, height: 1080,
        });
        let has_audio = s.audio_enabled;

        (child, start_time, clip_path, region, has_audio)
    };

    // Stop FFmpeg gracefully
    screen::stop_capture(&mut child).await
        .map_err(|e| format!("Failed to stop capture: {}", e))?;

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let clip_id = clip_path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Generate thumbnail
    let thumbnail_path = clip_path.with_extension("thumb.png");
    let thumb = if screen::generate_thumbnail(&clip_path, &thumbnail_path).is_ok() {
        Some(thumbnail_path)
    } else {
        None
    };

    let clip = Clip {
        id: clip_id,
        path: clip_path,
        duration_ms,
        region,
        has_audio,
        thumbnail_path: thumb,
    };

    // Add clip to state
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        // Add a default transition if there's already at least one clip
        if !s.clips.is_empty() {
            s.transitions.push(Transition::default());
        }
        s.clips.push(clip.clone());
    }

    Ok(clip)
}

pub fn cancel(state: &Mutex<AppState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = s.ffmpeg_process.take() {
        // Kill the process
        let _ = child.start_kill();
    }

    // Clean up the temp file
    if let Some(ref path) = s.current_clip_path.take() {
        let _ = std::fs::remove_file(path);
    }

    s.recording_start = None;
    s.recording_state = RecordingState::Idle;

    Ok(())
}
