use crate::types::{Clip, Transition, TransitionType};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tauri::{AppHandle, Emitter};

const TRANSITION_DURATION: f64 = 0.5;

fn ffmpeg_path() -> PathBuf {
    ffmpeg_sidecar::paths::ffmpeg_path()
}

fn xfade_name(t: &TransitionType) -> &'static str {
    match t {
        TransitionType::Fade => "fade",
        TransitionType::FadeBlack => "fadeblack",
        TransitionType::FadeWhite => "fadewhite",
        TransitionType::Dissolve => "dissolve",
        TransitionType::Zoom => "zoomin",
        TransitionType::Slide => "slideleft",
        TransitionType::SlideRight => "slideright",
        TransitionType::SlideUp => "slideup",
        TransitionType::SlideDown => "slidedown",
        TransitionType::WipeLeft => "wipeleft",
        TransitionType::WipeRight => "wiperight",
        TransitionType::WipeUp => "wipeup",
        TransitionType::WipeDown => "wipedown",
        TransitionType::Pixelize => "pixelize",
        TransitionType::CircleOpen => "circleopen",
        TransitionType::CircleClose => "circleclose",
        TransitionType::Radial => "radial",
        TransitionType::SmoothLeft => "smoothleft",
        TransitionType::SmoothRight => "smoothright",
    }
}

fn ffprobe_path() -> PathBuf {
    let name = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };
    ffmpeg_path().with_file_name(name)
}

/// Probe video duration in seconds using ffprobe
async fn probe_duration(path: &PathBuf) -> Result<f64> {
    if !path.exists() {
        anyhow::bail!("Clip file not found: {:?}", path);
    }
    let ffprobe = ffprobe_path();
    let output = Command::new(&ffprobe)
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            &path.to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .context("Failed to run ffprobe")?;

    let s = String::from_utf8_lossy(&output.stdout);
    s.trim().parse::<f64>().context("Failed to parse duration")
}

/// Build and run the FFmpeg export command with xfade transitions
pub async fn export_mp4(
    clips: &[Clip],
    transitions: &[Transition],
    output_path: &PathBuf,
    app: &AppHandle,
    watermark: bool,
) -> Result<()> {
    if clips.is_empty() {
        anyhow::bail!("No clips to export");
    }

    // Validate all clip files exist
    for (i, clip) in clips.iter().enumerate() {
        if !clip.path.exists() {
            anyhow::bail!("Clip {} file not found: {:?}", i + 1, clip.path);
        }
    }

    // Single clip: just copy
    if clips.len() == 1 {
        return export_single_clip(&clips[0], output_path, app, watermark).await;
    }

    // Get durations for all clips
    let mut durations = Vec::new();
    for clip in clips {
        let d = probe_duration(&clip.path).await?;
        durations.push(d);
    }

    // Find max resolution for scaling
    let max_w = clips.iter().map(|c| c.region.width).max().unwrap_or(1920);
    let max_h = clips.iter().map(|c| c.region.height).max().unwrap_or(1080);
    // Ensure even dimensions
    let max_w = (max_w / 2) * 2;
    let max_h = (max_h / 2) * 2;

    // Build FFmpeg args
    let mut args: Vec<String> = Vec::new();

    // Input files
    for clip in clips {
        args.push("-i".into());
        args.push(clip.path.to_string_lossy().to_string());
    }

    // Build filter_complex
    let mut filter = build_filter_complex(clips.len(), &durations, transitions, max_w, max_h);

    let final_label = if clips.len() == 2 {
        "[v0]".to_string()
    } else {
        format!("[v{}]", clips.len() - 2)
    };

    // Add watermark drawtext to the final output
    if watermark {
        let wm_label = "[final]";
        filter.push_str(&format!(
            ";{}drawtext=text='ClipFlow':fontsize=28:fontcolor=white@0.7:shadowcolor=black@0.5:shadowx=2:shadowy=2:x=w-tw-20:y=h-th-16{}",
            final_label, wm_label
        ));
        args.push("-filter_complex".into());
        args.push(filter);
        args.push("-map".into());
        args.push(wm_label.to_string());
    } else {
        args.push("-filter_complex".into());
        args.push(filter);
        args.push("-map".into());
        args.push(final_label);
    }

    // Output settings
    args.extend([
        "-c:v", "libx264",
        "-preset", "medium",
        "-crf", "20",
        "-pix_fmt", "yuv420p",
        "-r", "30",
        "-an", // no audio for now
        "-y",
    ].iter().map(|s| s.to_string()));

    args.push(output_path.to_string_lossy().to_string());

    // Log the command for debugging
    let ffmpeg = ffmpeg_path();
    eprintln!("[export] FFmpeg path: {:?}", ffmpeg);
    eprintln!("[export] Output: {:?}", output_path);
    eprintln!("[export] Args: {:?}", args);

    // Run FFmpeg with progress tracking
    let mut child = Command::new(&ffmpeg)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start FFmpeg export")?;

    // Parse stderr for progress
    let total_duration: f64 = durations.iter().sum::<f64>()
        - (transitions.len() as f64 * TRANSITION_DURATION);

    let mut stderr_log = String::new();
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("[ffmpeg] {}", line);
            stderr_log.push_str(&line);
            stderr_log.push('\n');
            if let Some(time_str) = extract_time(&line) {
                let progress = (time_str / total_duration * 100.0).min(100.0);
                let _ = app.emit("export-progress", progress as u32);
            }
        }
    }

    let status = child.wait().await.context("FFmpeg export failed")?;
    if !status.success() {
        eprintln!("[export] FFmpeg stderr:\n{}", stderr_log);
        anyhow::bail!("FFmpeg exited with code {:?}. Stderr: {}", status.code(), stderr_log.chars().take(500).collect::<String>());
    }

    eprintln!("[export] Export completed successfully");
    let _ = app.emit("export-progress", 100u32);
    Ok(())
}

async fn export_single_clip(
    clip: &Clip,
    output_path: &PathBuf,
    app: &AppHandle,
    watermark: bool,
) -> Result<()> {
    let _ = app.emit("export-progress", 10u32);

    let ffmpeg = ffmpeg_path();
    eprintln!("[export] Single clip export: {:?} -> {:?}, watermark={}", clip.path, output_path, watermark);

    let mut cmd_args: Vec<String> = vec![
        "-i".into(), clip.path.to_string_lossy().to_string(),
    ];

    if watermark {
        cmd_args.extend([
            "-vf".into(),
            "drawtext=text='ClipFlow':fontsize=28:fontcolor=white@0.7:shadowcolor=black@0.5:shadowx=2:shadowy=2:x=w-tw-20:y=h-th-16".into(),
        ]);
    }

    cmd_args.extend([
        "-c:v", "libx264",
        "-preset", "medium",
        "-crf", "20",
        "-pix_fmt", "yuv420p",
        "-r", "30",
        "-an",
        "-y",
    ].iter().map(|s| s.to_string()));
    cmd_args.push(output_path.to_string_lossy().to_string());

    let output = Command::new(&ffmpeg)
        .args(&cmd_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to export single clip")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[export] Single clip FFmpeg failed:\n{}", stderr);
        anyhow::bail!("FFmpeg exited with code {:?}. Stderr: {}", output.status.code(), stderr.chars().take(500).collect::<String>());
    }

    eprintln!("[export] Single clip export completed");
    let _ = app.emit("export-progress", 100u32);
    Ok(())
}

fn build_filter_complex(
    n: usize,
    durations: &[f64],
    transitions: &[Transition],
    max_w: u32,
    max_h: u32,
) -> String {
    let mut filters = Vec::new();

    // Scale all inputs to the same resolution
    for i in 0..n {
        filters.push(format!(
            "[{i}]scale={max_w}:{max_h}:force_original_aspect_ratio=decrease,pad={max_w}:{max_h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30[s{i}]"
        ));
    }

    // Chain xfade transitions
    let mut prev_label = "[s0]".to_string();
    let mut cumulative_offset: f64 = 0.0;

    for i in 0..(n - 1) {
        cumulative_offset += durations[i] - TRANSITION_DURATION;

        let transition_type = transitions
            .get(i)
            .map(|t| &t.transition_type)
            .unwrap_or(&TransitionType::Fade);

        let out_label = format!("[v{i}]");
        let next_input = format!("[s{}]", i + 1);

        filters.push(format!(
            "{prev_label}{next_input}xfade=transition={}:duration={TRANSITION_DURATION}:offset={cumulative_offset:.3}{out_label}",
            xfade_name(transition_type),
        ));

        prev_label = out_label;
    }

    filters.join(";")
}

/// Build and run a fast preview (ultrafast, lower res)
pub async fn preview_mp4(
    clips: &[Clip],
    transitions: &[Transition],
    output_path: &PathBuf,
    app: &AppHandle,
) -> Result<()> {
    if clips.is_empty() {
        anyhow::bail!("No clips to preview");
    }

    if clips.len() == 1 {
        return preview_single_clip(&clips[0], output_path, app).await;
    }

    let mut durations = Vec::new();
    for clip in clips {
        let d = probe_duration(&clip.path).await?;
        durations.push(d);
    }

    let max_w = clips.iter().map(|c| c.region.width).max().unwrap_or(1920);
    let max_h = clips.iter().map(|c| c.region.height).max().unwrap_or(1080);
    // Half resolution for preview, ensure even
    let prev_w = ((max_w / 2) / 2) * 2;
    let prev_h = ((max_h / 2) / 2) * 2;
    let prev_w = prev_w.max(320);
    let prev_h = prev_h.max(240);

    let mut args: Vec<String> = Vec::new();
    for clip in clips {
        args.push("-i".into());
        args.push(clip.path.to_string_lossy().to_string());
    }

    let filter = build_filter_complex(clips.len(), &durations, transitions, prev_w, prev_h);
    args.push("-filter_complex".into());
    args.push(filter);

    let final_label = if clips.len() == 2 {
        "[v0]".to_string()
    } else {
        format!("[v{}]", clips.len() - 2)
    };
    args.push("-map".into());
    args.push(final_label);

    args.extend([
        "-c:v", "libx264",
        "-preset", "ultrafast",
        "-crf", "30",
        "-pix_fmt", "yuv420p",
        "-r", "24",
        "-an",
        "-y",
    ].iter().map(|s| s.to_string()));
    args.push(output_path.to_string_lossy().to_string());

    let ffmpeg = ffmpeg_path();
    eprintln!("[preview] Args: {:?}", args);

    let mut child = Command::new(&ffmpeg)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start FFmpeg preview")?;

    let total_duration: f64 = durations.iter().sum::<f64>()
        - (transitions.len() as f64 * TRANSITION_DURATION);

    let mut stderr_log = String::new();
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("[ffmpeg-preview] {}", line);
            stderr_log.push_str(&line);
            stderr_log.push('\n');
            if let Some(time_str) = extract_time(&line) {
                let progress = (time_str / total_duration * 100.0).min(100.0);
                let _ = app.emit("preview-progress", progress as u32);
            }
        }
    }

    let status = child.wait().await.context("FFmpeg preview failed")?;
    if !status.success() {
        anyhow::bail!("Preview failed: {}", stderr_log.chars().take(500).collect::<String>());
    }

    let _ = app.emit("preview-progress", 100u32);
    Ok(())
}

async fn preview_single_clip(
    clip: &Clip,
    output_path: &PathBuf,
    app: &AppHandle,
) -> Result<()> {
    let _ = app.emit("preview-progress", 10u32);
    let ffmpeg = ffmpeg_path();

    let prev_w = ((clip.region.width / 2) / 2) * 2;
    let prev_h = ((clip.region.height / 2) / 2) * 2;
    let scale = format!("scale={}:{}", prev_w.max(320), prev_h.max(240));

    let output = Command::new(&ffmpeg)
        .args([
            "-i", &clip.path.to_string_lossy(),
            "-vf", &scale,
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-crf", "30",
            "-pix_fmt", "yuv420p",
            "-r", "24",
            "-an",
            "-y",
            &output_path.to_string_lossy(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to preview single clip")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Preview failed: {}", stderr.chars().take(500).collect::<String>());
    }

    let _ = app.emit("preview-progress", 100u32);
    Ok(())
}

/// Extract time in seconds from FFmpeg stderr output like "time=00:01:23.45"
fn extract_time(line: &str) -> Option<f64> {
    let time_prefix = "time=";
    let idx = line.find(time_prefix)?;
    let time_str = &line[idx + time_prefix.len()..];
    let end = time_str.find(' ').unwrap_or(time_str.len());
    let time_str = &time_str[..end];

    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let hours: f64 = parts[0].parse().ok()?;
        let minutes: f64 = parts[1].parse().ok()?;
        let seconds: f64 = parts[2].parse().ok()?;
        Some(hours * 3600.0 + minutes * 60.0 + seconds)
    } else {
        None
    }
}
