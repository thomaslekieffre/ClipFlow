use crate::capture::screen;
use crate::state::{AppState, AudioCaptureHandle};
use crate::types::{AudioSource, Clip, RecordingState, Region, Transition};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::Instant;
use tauri::AppHandle;

const FRAMERATE: u32 = 30;

pub fn start(state: &Mutex<AppState>, app: &AppHandle) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if s.recording_state != RecordingState::Idle {
        return Err("Enregistrement déjà en cours".into());
    }

    // Ensure temp dir exists
    std::fs::create_dir_all(&s.temp_dir).map_err(|e| e.to_string())?;

    // Reset segment tracking
    s.recording_segments = Vec::new();
    s.pause_accumulated_ms = 0;
    s.segment_index = 0;

    let clip_id = uuid::Uuid::new_v4().to_string();
    let clip_path = s.temp_dir.join(format!("{}.mp4", clip_id));

    let child = if let Some(ref region) = s.current_region {
        screen::start_capture(region, &clip_path, FRAMERATE)
    } else {
        screen::start_fullscreen_capture(&clip_path, FRAMERATE)
    }
    .map_err(|e| format!("Failed to start capture: {}", e))?;

    s.ffmpeg_process = Some(child);
    let start_time = Instant::now();
    s.recording_start = Some(start_time);
    s.current_clip_path = Some(clip_path);
    s.recording_state = RecordingState::Recording;

    // Start audio capture
    start_audio_captures(&mut s, &clip_id);

    // Start keystroke capture (with live emission if enabled)
    if s.keystroke_enabled {
        match crate::capture::keystroke::start_capture_with_emitter(start_time, app.clone()) {
            Ok(handle) => s.keystroke_handle = Some(handle),
            Err(e) => eprintln!("[recording] Failed to start keystroke capture: {}", e),
        }
    }

    // Start cursor tracking
    if s.cursor_zoom_enabled {
        let region = s.current_region.clone().unwrap_or(Region {
            x: 0, y: 0, width: 1920, height: 1080,
        });
        s.cursor_handle = Some(crate::capture::cursor::start_tracking(&region, start_time));
    }

    Ok(())
}

fn start_audio_captures(s: &mut AppState, clip_id: &str) {
    let audio_source = s.audio_source;
    if audio_source == AudioSource::None {
        return;
    }

    s.audio_temp_paths.clear();
    s.audio_handles.clear();

    if matches!(audio_source, AudioSource::System | AudioSource::Both) {
        let path = s.temp_dir.join(format!("{}_system.wav", clip_id));
        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        match crate::capture::audio::start_system_capture(&path, stop_flag.clone()) {
            Ok(handle) => {
                s.audio_handles.push(AudioCaptureHandle {
                    join_handle: Some(handle),
                    stop_flag,
                });
                s.audio_temp_paths.push(path);
            }
            Err(e) => eprintln!("[recording] Failed to start system audio: {}", e),
        }
    }

    if matches!(audio_source, AudioSource::Microphone | AudioSource::Both) {
        let path = s.temp_dir.join(format!("{}_mic.wav", clip_id));
        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mic_name = s.selected_mic.as_deref();
        match crate::capture::audio::start_mic_capture_device(&path, stop_flag.clone(), mic_name) {
            Ok(handle) => {
                s.audio_handles.push(AudioCaptureHandle {
                    join_handle: Some(handle),
                    stop_flag,
                });
                s.audio_temp_paths.push(path);
            }
            Err(e) => eprintln!("[recording] Failed to start mic audio: {}", e),
        }
    }
}

fn stop_audio_captures(audio_handles: &mut Vec<AudioCaptureHandle>) {
    for handle in audio_handles.drain(..) {
        handle.stop_flag.store(true, Ordering::Relaxed);
        if let Some(h) = handle.join_handle {
            let _ = h.join();
        }
    }
}

pub async fn pause(state: &Mutex<AppState>) -> Result<(), String> {
    let (mut child, start_time, segment_path, mut audio_handles) = {
        let mut s = state.lock().map_err(|e| e.to_string())?;

        if s.recording_state != RecordingState::Recording {
            return Err("Pas d'enregistrement en cours".into());
        }

        let child = s.ffmpeg_process.take()
            .ok_or("Processus FFmpeg introuvable")?;
        let start_time = s.recording_start.take()
            .ok_or("Heure de début introuvable")?;
        let segment_path = s.current_clip_path.clone()
            .ok_or("Chemin du clip introuvable")?;
        let audio_handles = std::mem::take(&mut s.audio_handles);

        s.recording_state = RecordingState::Paused;
        (child, start_time, segment_path, audio_handles)
    };

    // Stop the current FFmpeg segment gracefully
    screen::stop_capture(&mut child).await
        .map_err(|e| format!("Failed to stop capture segment: {}", e))?;

    // Stop audio captures during pause to avoid timing issues
    stop_audio_captures(&mut audio_handles);

    // Track accumulated time and save segment
    let segment_ms = start_time.elapsed().as_millis() as u64;
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.pause_accumulated_ms += segment_ms;
        s.recording_segments.push(segment_path);
        s.segment_index += 1;
    }

    Ok(())
}

pub fn resume(state: &Mutex<AppState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if s.recording_state != RecordingState::Paused {
        return Err("L'enregistrement n'est pas en pause".into());
    }

    // Create a new segment file
    let clip_id = s.current_clip_path.as_ref()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let segment_path = s.temp_dir.join(format!("{}_seg{}.mp4", clip_id, s.segment_index));

    let child = if let Some(ref region) = s.current_region {
        screen::start_capture(region, &segment_path, FRAMERATE)
    } else {
        screen::start_fullscreen_capture(&segment_path, FRAMERATE)
    }
    .map_err(|e| format!("Failed to resume capture: {}", e))?;

    s.ffmpeg_process = Some(child);
    s.recording_start = Some(Instant::now());
    s.current_clip_path = Some(segment_path);
    s.recording_state = RecordingState::Recording;

    // Restart audio captures
    // Use the base clip_id (without _seg suffix) for audio file naming
    let base_id = clip_id.split("_seg").next().unwrap_or(&clip_id).to_string();
    let audio_id = format!("{}_seg{}", base_id, s.segment_index);
    start_audio_captures(&mut s, &audio_id);

    Ok(())
}

pub async fn stop(state: &Mutex<AppState>) -> Result<Clip, String> {
    // First lock: extract all handles and state
    let (
        mut child, start_time, clip_path, region,
        segments, accumulated_ms,
        mut audio_handles, audio_temp_paths,
        keystroke_handle, cursor_handle,
    ) = {
        let mut s = state.lock().map_err(|e| e.to_string())?;

        if s.recording_state != RecordingState::Recording && s.recording_state != RecordingState::Paused {
            return Err("Pas d'enregistrement en cours".into());
        }

        let child = s.ffmpeg_process.take();
        let start_time = s.recording_start.take();
        let clip_path = s.current_clip_path.take()
            .ok_or("Chemin du clip introuvable")?;

        // Mark idle immediately to prevent concurrent stop attempts
        s.recording_state = RecordingState::Idle;
        let region = s.current_region.clone().unwrap_or(Region {
            x: 0, y: 0, width: 1920, height: 1080,
        });
        let segments = std::mem::take(&mut s.recording_segments);
        let accumulated_ms = s.pause_accumulated_ms;

        // Take capture handles
        let audio_handles = std::mem::take(&mut s.audio_handles);
        let audio_temp_paths = std::mem::take(&mut s.audio_temp_paths);
        let keystroke_handle = s.keystroke_handle.take();
        let cursor_handle = s.cursor_handle.take();

        (
            child, start_time, clip_path, region,
            segments, accumulated_ms,
            audio_handles, audio_temp_paths,
            keystroke_handle, cursor_handle,
        )
    };
    // Mutex is now unlocked — safe to do blocking operations

    // Stop FFmpeg gracefully if still running (not paused)
    if let Some(ref mut c) = child {
        screen::stop_capture(c).await
            .map_err(|e| format!("Failed to stop capture: {}", e))?;
    }

    // Stop audio captures
    stop_audio_captures(&mut audio_handles);

    // Stop keystroke capture
    let keystroke_events = if let Some(mut handle) = keystroke_handle {
        crate::capture::keystroke::stop_capture(&mut handle)
    } else {
        Vec::new()
    };

    // Stop cursor tracking
    let cursor_positions = if let Some(mut handle) = cursor_handle {
        crate::capture::cursor::stop_tracking(&mut handle)
    } else {
        Vec::new()
    };

    let last_segment_ms = start_time.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);
    let total_duration_ms = accumulated_ms + last_segment_ms;

    // Determine final output path
    let final_path = if segments.is_empty() {
        // No pause was used — single file, just use clip_path as-is
        if !clip_path.exists() {
            eprintln!("[recording] Clip file not created: {:?}. FFmpeg capture may have failed.", clip_path);
            return Err(format!("Capture échouée : le fichier vidéo n'a pas été créé. Vérifiez que la zone de capture est valide."));
        }
        clip_path
    } else {
        // Multiple segments — concat them
        let mut all_segments = segments;
        // Add the last segment (current clip_path) if it was recording (not paused at stop)
        if child.is_some() {
            all_segments.push(clip_path.clone());
        }

        let concat_output = clip_path.with_extension("concat.mp4");
        concat_segments(&all_segments, &concat_output).await?;

        // Cleanup individual segment files
        for seg in &all_segments {
            let _ = std::fs::remove_file(seg);
        }

        // Rename to final path
        let final_out = if clip_path.exists() {
            concat_output
        } else {
            let _ = std::fs::rename(&concat_output, &clip_path);
            clip_path
        };
        final_out
    };

    let clip_id = final_path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
        .replace("_seg", "")
        .replace(".concat", "");

    // Generate thumbnail
    let thumbnail_path = final_path.with_extension("thumb.png");
    let thumb = if screen::generate_thumbnail(&final_path, &thumbnail_path).is_ok() {
        Some(thumbnail_path)
    } else {
        None
    };

    // Collect audio paths (only existing files)
    let audio_paths: Vec<String> = audio_temp_paths.iter()
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let has_audio = !audio_paths.is_empty();

    let clip = Clip {
        id: clip_id.clone(),
        path: final_path,
        duration_ms: total_duration_ms,
        region,
        has_audio,
        thumbnail_path: thumb,
        trim_start_ms: 0,
        trim_end_ms: 0,
        audio_paths,
    };

    // Second lock: store clip and associated data
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        // Add a default transition if there's already at least one clip
        if !s.clips.is_empty() {
            s.transitions.push(Transition::default());
        }
        s.clips.push(clip.clone());
        // Reset pause state
        s.pause_accumulated_ms = 0;
        s.segment_index = 0;

        // Store keystroke and cursor data
        if !keystroke_events.is_empty() {
            s.clip_keystrokes.insert(clip_id.clone(), keystroke_events);
        }
        if !cursor_positions.is_empty() {
            s.clip_cursor_positions.insert(clip_id, cursor_positions);
        }
    }

    Ok(clip)
}

pub fn cancel(state: &Mutex<AppState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = s.ffmpeg_process.take() {
        let _ = child.start_kill();
    }

    // Clean up the current temp file
    if let Some(ref path) = s.current_clip_path.take() {
        let _ = std::fs::remove_file(path);
    }

    // Clean up any segments from pauses
    for seg in s.recording_segments.drain(..) {
        let _ = std::fs::remove_file(&seg);
    }

    // Stop audio captures and clean up temp files
    for handle in s.audio_handles.drain(..) {
        handle.stop_flag.store(true, Ordering::Relaxed);
        if let Some(h) = handle.join_handle {
            let _ = h.join();
        }
    }
    for path in s.audio_temp_paths.drain(..) {
        let _ = std::fs::remove_file(&path);
    }

    // Stop keystroke capture (discard data)
    if let Some(mut handle) = s.keystroke_handle.take() {
        let _ = crate::capture::keystroke::stop_capture(&mut handle);
    }

    // Stop cursor tracking (discard data)
    if let Some(mut handle) = s.cursor_handle.take() {
        let _ = crate::capture::cursor::stop_tracking(&mut handle);
    }

    s.recording_start = None;
    s.recording_state = RecordingState::Idle;
    s.pause_accumulated_ms = 0;
    s.segment_index = 0;

    Ok(())
}

/// Concatenate multiple video segments using FFmpeg concat demuxer
async fn concat_segments(segments: &[PathBuf], output: &PathBuf) -> Result<(), String> {
    if segments.is_empty() {
        return Err("Aucun segment à concaténer".into());
    }
    if segments.len() == 1 {
        std::fs::copy(&segments[0], output).map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Create concat list file
    let list_path = output.with_extension("txt");
    let mut list_content = String::new();
    for seg in segments {
        if seg.exists() {
            list_content.push_str(&format!("file '{}'\n", seg.to_string_lossy().replace('\'', "'\\''")));
        }
    }
    std::fs::write(&list_path, &list_content).map_err(|e| e.to_string())?;

    let result = crate::ffmpeg_command()
        .args([
            "-f", "concat",
            "-safe", "0",
            "-i", &list_path.to_string_lossy(),
            "-c", "copy",
            "-y",
            &output.to_string_lossy(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run concat: {}", e))?;

    let _ = std::fs::remove_file(&list_path);

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Échec de la concaténation : {}", stderr.chars().take(500).collect::<String>()));
    }

    Ok(())
}
