use crate::types::{
    Annotation, AnnotationKind, Clip, CursorPosition, ExportQuality,
    KeystrokeEvent, Subtitle, SubtitlePosition, Transition, TransitionType,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tauri::{AppHandle, Emitter};

const TRANSITION_DURATION: f64 = 0.5;
const CURSOR_ZOOM: f64 = 1.15;

fn ffmpeg_path() -> PathBuf {
    crate::ffmpeg_bin()
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
        TransitionType::Cut => "fade",
    }
}

fn ffprobe_path() -> PathBuf {
    crate::ffprobe_bin()
}

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

fn effective_duration(clip: &Clip, probed_duration: f64) -> f64 {
    let start = clip.trim_start_ms as f64 / 1000.0;
    let end = if clip.trim_end_ms > 0 {
        clip.trim_end_ms as f64 / 1000.0
    } else {
        probed_duration
    };
    (end - start).max(0.1)
}

const WATERMARK_FILTER: &str = "drawtext=text='ClipFlow':fontsize=28:fontcolor=white@0.7:shadowcolor=black@0.5:shadowx=2:shadowy=2:x=w-tw-20:y=h-th-16";

fn all_cuts(transitions: &[Transition]) -> bool {
    transitions.iter().all(|t| t.transition_type == TransitionType::Cut)
}

// ──────────────────────────────── Helper: text escaping ────────────────────────────────

fn escape_drawtext(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\'', "'\\\\\\''")
        .replace(':', "\\:")
        .replace('%', "%%")
}

// ──────────────────────────────── Helper: piecewise linear interpolation ────────────────

/// Build FFmpeg expression for piecewise linear interpolation over keyframes.
/// Keyframes: (time_seconds, value). Commas escaped for filter_complex.
fn build_piecewise_lerp(keyframes: &[(f64, f64)]) -> String {
    if keyframes.is_empty() {
        return "0".to_string();
    }
    if keyframes.len() == 1 {
        return format!("{:.4}", keyframes[0].1);
    }

    // Build from the end (last value is the fallback)
    let mut expr = format!("{:.4}", keyframes.last().unwrap().1);

    for i in (0..keyframes.len() - 1).rev() {
        let (t0, v0) = keyframes[i];
        let (t1, _v1) = keyframes[i + 1];
        let dt = t1 - t0;
        if dt < 0.001 { continue; }

        let v_next = keyframes[i + 1].1;
        let dv = v_next - v0;
        let lerp = if dv.abs() < 0.0001 {
            format!("{:.4}", v0)
        } else {
            format!("{:.4}+{:.4}*(t-{:.2})/{:.2}", v0, dv, t0, dt)
        };

        expr = format!("if(lt(t\\,{:.2})\\,{}\\,{})", t1, lerp, expr);
    }

    expr
}

// ──────────────────────────────── Cursor Zoom ────────────────────────────────

/// Build a crop+scale filter that follows cursor position with smooth panning.
fn build_cursor_zoom_filter(
    positions: &[CursorPosition],
    trim_start_ms: u64,
    width: u32,
    height: u32,
) -> Option<String> {
    if positions.is_empty() {
        return None;
    }

    let trim_s = trim_start_ms as f64 / 1000.0;

    // Check if cursor actually moves significantly (>10% of screen in X or Y)
    let min_x = positions.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = positions.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = positions.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = positions.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    if (max_x - min_x) < 0.10 && (max_y - min_y) < 0.10 {
        return None; // Cursor barely moved — skip zoom
    }

    let half_inv = 1.0 / (2.0 * CURSOR_ZOOM);
    let max_frac = 1.0 - 1.0 / CURSOR_ZOOM;

    // Group positions by second, average within each second
    let mut second_groups: std::collections::BTreeMap<i64, Vec<(f64, f64)>> = std::collections::BTreeMap::new();
    for pos in positions {
        let t = pos.timestamp_ms as f64 / 1000.0 - trim_s;
        if t < 0.0 { continue; }
        let sec = t.floor() as i64;
        second_groups.entry(sec).or_default().push((pos.x, pos.y));
    }

    let mut kf_x: Vec<(f64, f64)> = Vec::new();
    let mut kf_y: Vec<(f64, f64)> = Vec::new();

    for (sec, group) in &second_groups {
        if kf_x.len() >= 30 { break; }
        let avg_x = group.iter().map(|p| p.0).sum::<f64>() / group.len() as f64;
        let avg_y = group.iter().map(|p| p.1).sum::<f64>() / group.len() as f64;
        let x = (avg_x - half_inv).clamp(0.0, max_frac);
        let y = (avg_y - half_inv).clamp(0.0, max_frac);
        kf_x.push((*sec as f64, x));
        kf_y.push((*sec as f64, y));
    }

    if kf_x.is_empty() {
        return None;
    }

    let x_expr = build_piecewise_lerp(&kf_x);
    let y_expr = build_piecewise_lerp(&kf_y);

    Some(format!(
        "crop=w=iw/{z:.1}:h=ih/{z:.1}:x='({x})*iw':y='({y})*ih':exact=1,scale={w}:{h}",
        z = CURSOR_ZOOM, x = x_expr, y = y_expr, w = width, h = height
    ))
}

// ──────────────────────────────── Annotations ────────────────────────────────

/// Convert annotation color (hex or name) to FFmpeg color string.
fn annotation_color_ffmpeg(color: &str) -> String {
    if color.starts_with('#') && color.len() == 7 {
        format!("0x{}", &color[1..])
    } else {
        color.to_string()
    }
}

/// Build drawbox / drawtext filters for annotations on a single clip.
fn build_annotation_draw_filters(
    annotations: &[Annotation],
    width: u32,
    height: u32,
) -> Vec<String> {
    let mut filters = Vec::new();

    for ann in annotations {
        let start_s = ann.start_ms as f64 / 1000.0;
        let end_s = ann.end_ms as f64 / 1000.0;
        let enable = format!("enable='between(t\\,{:.3}\\,{:.3})'", start_s, end_s);

        let px = (ann.x * width as f64) as i32;
        let py = (ann.y * height as f64) as i32;
        let pw = (ann.width * width as f64).max(4.0) as u32;
        let ph = (ann.height * height as f64).max(4.0) as u32;
        let stroke = ann.stroke_width.max(2.0) as u32;
        let color = annotation_color_ffmpeg(&ann.color);

        match ann.kind {
            AnnotationKind::Rectangle => {
                filters.push(format!(
                    "drawbox=x={px}:y={py}:w={pw}:h={ph}:color={color}@0.8:t={stroke}:{enable}"
                ));
            }
            AnnotationKind::Circle => {
                // Approximate circle with a square outline
                let cx = px + pw as i32 / 2;
                let cy = py + ph as i32 / 2;
                let r = pw.min(ph) / 2;
                let bx = (cx - r as i32).max(0) as u32;
                let by = (cy - r as i32).max(0) as u32;
                filters.push(format!(
                    "drawbox=x={bx}:y={by}:w={d}:h={d}:color={color}@0.8:t={stroke}:{enable}",
                    d = r * 2
                ));
            }
            AnnotationKind::Text => {
                if let Some(ref text) = ann.text {
                    let escaped = escape_drawtext(text);
                    let fontsize = (ann.height * height as f64 * 0.7).max(16.0) as u32;
                    filters.push(format!(
                        "drawtext=text='{escaped}':x={px}:y={py}:fontsize={fontsize}:fontcolor={color}:{enable}"
                    ));
                }
            }
            AnnotationKind::Arrow | AnnotationKind::Freehand => {
                // Arrow: draw a thick line approximation using drawbox
                if ann.kind == AnnotationKind::Arrow {
                    // Draw the shaft as a thin horizontal/diagonal box
                    let thick = stroke.max(3);
                    filters.push(format!(
                        "drawbox=x={px}:y={}:w={pw}:h={thick}:color={color}@0.8:t=fill:{enable}",
                        py + ph as i32 / 2 - thick as i32 / 2
                    ));
                }
                // Freehand: draw connected segments as small boxes along the path
                if ann.kind == AnnotationKind::Freehand {
                    if let Some(ref points) = ann.points {
                        let dot_size = stroke.max(3);
                        // Draw every 3rd point to keep filter manageable
                        for pt in points.iter().step_by(3).take(50) {
                            let dx = (pt.0 * width as f64) as i32;
                            let dy = (pt.1 * height as f64) as i32;
                            filters.push(format!(
                                "drawbox=x={dx}:y={dy}:w={dot_size}:h={dot_size}:color={color}@0.8:t=fill:{enable}"
                            ));
                        }
                    }
                }
            }
        }
    }

    filters
}

// ──────────────────────────────── Keystrokes / Subtitles ────────────────────────────────

fn build_keystroke_filters(
    events: &[KeystrokeEvent],
    time_offset: f64,
    trim_start_ms: u64,
) -> Vec<String> {
    let mut filters = Vec::new();
    let trim_offset = trim_start_ms as f64 / 1000.0;

    let events_to_use = if events.len() > 150 { &events[..150] } else { events };

    for event in events_to_use {
        let t = event.timestamp_ms as f64 / 1000.0 - trim_offset + time_offset;
        if t < 0.0 { continue; }
        let end_t = t + 1.5;
        let escaped = escape_drawtext(&event.key_name);
        filters.push(format!(
            "drawtext=text='{escaped}':fontsize=28:fontcolor=white:box=1:boxcolor=black@0.7:boxborderw=10:x=(w-tw)/2:y=h-60:enable='between(t\\,{t:.3}\\,{end_t:.3})'"
        ));
    }
    filters
}

fn build_subtitle_filters(subtitles: &[Subtitle]) -> Vec<String> {
    let mut filters = Vec::new();

    for sub in subtitles {
        let start_s = sub.start_ms as f64 / 1000.0;
        let end_s = sub.end_ms as f64 / 1000.0;
        let escaped = escape_drawtext(&sub.text);

        let y_expr = match sub.position {
            SubtitlePosition::Top => "30",
            SubtitlePosition::Center => "(h-th)/2",
            SubtitlePosition::Bottom => "h-th-30",
        };
        let fontsize = if sub.font_size > 0 { sub.font_size } else { 32 };
        let color = annotation_color_ffmpeg(&sub.color);

        filters.push(format!(
            "drawtext=text='{escaped}':fontsize={fontsize}:fontcolor={color}:box=1:boxcolor=black@0.6:boxborderw=8:x=(w-tw)/2:y={y_expr}:enable='between(t\\,{start_s:.3}\\,{end_s:.3})'"
        ));
    }
    filters
}

// ──────────────────────────────── Audio helpers ────────────────────────────────

fn build_audio_concat_filter(
    audio_input_map: &[(usize, Vec<usize>)],
    eff_durations: &[f64],
) -> String {
    let mut filters = Vec::new();
    let mut has_any = false;
    let n = audio_input_map.len();

    for (ci, (_clip_idx, indices)) in audio_input_map.iter().enumerate() {
        if indices.is_empty() {
            let dur = eff_durations.get(ci).copied().unwrap_or(1.0);
            filters.push(format!(
                "anullsrc=channel_layout=stereo:sample_rate=44100,atrim=duration={dur:.3}[a{ci}]"
            ));
        } else if indices.len() == 1 {
            filters.push(format!("[{}:a]anull[a{ci}]", indices[0]));
            has_any = true;
        } else {
            let mix_inputs: String = indices.iter().map(|idx| format!("[{}:a]", idx)).collect();
            filters.push(format!(
                "{mix_inputs}amix=inputs={}:duration=first[a{ci}]",
                indices.len()
            ));
            has_any = true;
        }
    }

    if !has_any {
        return String::new();
    }

    let audio_labels: String = (0..n).map(|i| format!("[a{i}]")).collect();
    filters.push(format!("{audio_labels}concat=n={n}:v=0:a=1[aout]"));
    filters.join(";")
}

// ──────────────────────────────── Per-clip filter chain ────────────────────────────────

/// Build the filter chain for a single clip inside filter_complex:
/// trim → scale+pad → cursor_zoom → annotations → [si]
fn build_clip_chain(
    i: usize,
    clip: &Clip,
    max_w: u32,
    max_h: u32,
    cursor_positions: Option<&Vec<CursorPosition>>,
    annotations: Option<&Vec<Annotation>>,
) -> String {
    let has_trim = clip.trim_start_ms > 0 || clip.trim_end_ms > 0;

    // 1. Trim
    let trim_part = if has_trim {
        let start_s = clip.trim_start_ms as f64 / 1000.0;
        let end_s = clip.trim_end_ms as f64 / 1000.0;
        if clip.trim_start_ms > 0 && clip.trim_end_ms > 0 {
            format!("trim=start={start_s:.3}:end={end_s:.3},setpts=PTS-STARTPTS,")
        } else if clip.trim_start_ms > 0 {
            format!("trim=start={start_s:.3},setpts=PTS-STARTPTS,")
        } else {
            format!("trim=end={end_s:.3},setpts=PTS-STARTPTS,")
        }
    } else {
        String::new()
    };

    // 2. Scale + pad + setsar + fps
    let mut chain = format!(
        "[{i}:v]{trim_part}scale={max_w}:{max_h}:force_original_aspect_ratio=decrease,pad={max_w}:{max_h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30"
    );

    // 3. Cursor zoom (crop + scale)
    if let Some(positions) = cursor_positions {
        if let Some(zoom_filter) = build_cursor_zoom_filter(positions, clip.trim_start_ms, max_w, max_h) {
            chain.push(',');
            chain.push_str(&zoom_filter);
        }
    }

    // 4. Annotations (drawbox, drawtext)
    if let Some(anns) = annotations {
        for f in build_annotation_draw_filters(anns, max_w, max_h) {
            chain.push(',');
            chain.push_str(&f);
        }
    }

    chain.push_str(&format!("[s{i}]"));
    chain
}

// ──────────────────────────────── Filter complex builder ────────────────────────────────

fn build_filter_complex_with_trim(
    clips: &[Clip],
    eff_durations: &[f64],
    transitions: &[Transition],
    max_w: u32,
    max_h: u32,
    clip_annotations: &HashMap<String, Vec<Annotation>>,
    clip_cursor_positions: &HashMap<String, Vec<CursorPosition>>,
) -> String {
    let n = clips.len();
    let mut filters = Vec::new();

    // Per-clip processing chains
    for i in 0..n {
        let clip = &clips[i];
        let cursor = clip_cursor_positions.get(&clip.id);
        let anns = clip_annotations.get(&clip.id);
        filters.push(build_clip_chain(i, clip, max_w, max_h, cursor, anns));
    }

    // Chain xfade / concat transitions
    let mut prev_label = "[s0]".to_string();
    let mut cumulative_offset: f64 = 0.0;

    for i in 0..(n - 1) {
        let transition_type = transitions
            .get(i)
            .map(|t| &t.transition_type)
            .unwrap_or(&TransitionType::Fade);

        if *transition_type == TransitionType::Cut {
            cumulative_offset += eff_durations[i];
        } else {
            cumulative_offset += eff_durations[i] - TRANSITION_DURATION;
        }

        let out_label = format!("[v{i}]");
        let next_input = format!("[s{}]", i + 1);

        if *transition_type == TransitionType::Cut {
            filters.push(format!(
                "{prev_label}{next_input}concat=n=2:v=1:a=0{out_label}"
            ));
        } else {
            filters.push(format!(
                "{prev_label}{next_input}xfade=transition={}:duration={TRANSITION_DURATION}:offset={cumulative_offset:.3}{out_label}",
                xfade_name(transition_type),
            ));
        }

        prev_label = out_label;
    }

    filters.join(";")
}

// ──────────────────────────────── Export MP4 (multi-clip) ────────────────────────────────

pub async fn export_mp4(
    clips: &[Clip],
    transitions: &[Transition],
    output_path: &PathBuf,
    app: &AppHandle,
    watermark: bool,
    quality: &ExportQuality,
    clip_keystrokes: &HashMap<String, Vec<KeystrokeEvent>>,
    subtitles: &[Subtitle],
    clip_annotations: &HashMap<String, Vec<Annotation>>,
    clip_cursor_positions: &HashMap<String, Vec<CursorPosition>>,
) -> Result<()> {
    if clips.is_empty() {
        anyhow::bail!("No clips to export");
    }
    for (i, clip) in clips.iter().enumerate() {
        if !clip.path.exists() {
            anyhow::bail!("Clip {} file not found: {:?}", i + 1, clip.path);
        }
    }

    if clips.len() == 1 {
        return export_single_clip(
            &clips[0], output_path, app, watermark, quality,
            clip_keystrokes, subtitles, clip_annotations, clip_cursor_positions,
        ).await;
    }

    let mut durations = Vec::new();
    for clip in clips {
        durations.push(probe_duration(&clip.path).await?);
    }
    let eff_durations: Vec<f64> = clips.iter().zip(durations.iter())
        .map(|(c, d)| effective_duration(c, *d))
        .collect();

    if all_cuts(transitions) {
        return export_with_concat(
            clips, &eff_durations, output_path, app, watermark, quality,
            clip_keystrokes, subtitles, clip_annotations, clip_cursor_positions,
        ).await;
    }

    let max_w = (clips.iter().map(|c| c.region.width).max().unwrap_or(1920) / 2) * 2;
    let max_h = (clips.iter().map(|c| c.region.height).max().unwrap_or(1080) / 2) * 2;

    let has_any_audio = clips.iter().any(|c| !c.audio_paths.is_empty());

    let mut args: Vec<String> = Vec::new();
    for clip in clips {
        args.push("-i".into());
        args.push(clip.path.to_string_lossy().to_string());
    }

    // Audio inputs
    let num_video_inputs = clips.len();
    let mut audio_input_map: Vec<(usize, Vec<usize>)> = Vec::new();
    if has_any_audio {
        let mut input_idx = num_video_inputs;
        for (ci, clip) in clips.iter().enumerate() {
            let mut indices = Vec::new();
            for audio_path in &clip.audio_paths {
                if std::path::Path::new(audio_path).exists() {
                    args.push("-i".into());
                    args.push(audio_path.clone());
                    indices.push(input_idx);
                    input_idx += 1;
                }
            }
            audio_input_map.push((ci, indices));
        }
    }

    let mut filter = build_filter_complex_with_trim(
        clips, &eff_durations, transitions, max_w, max_h,
        clip_annotations, clip_cursor_positions,
    );

    let video_final_label = if clips.len() == 2 {
        "[v0]".to_string()
    } else {
        format!("[v{}]", clips.len() - 2)
    };

    // Global overlays: keystrokes, subtitles, watermark
    let mut overlay_filters = Vec::new();
    let mut cumulative_time = 0.0;
    for (i, clip) in clips.iter().enumerate() {
        if let Some(events) = clip_keystrokes.get(&clip.id) {
            overlay_filters.extend(build_keystroke_filters(events, cumulative_time, clip.trim_start_ms));
        }
        if i < transitions.len() {
            if transitions[i].transition_type == TransitionType::Cut {
                cumulative_time += eff_durations[i];
            } else {
                cumulative_time += eff_durations[i] - TRANSITION_DURATION;
            }
        } else {
            cumulative_time += eff_durations[i];
        }
    }
    overlay_filters.extend(build_subtitle_filters(subtitles));
    if watermark {
        overlay_filters.push(WATERMARK_FILTER.to_string());
    }

    let output_label = if overlay_filters.is_empty() {
        video_final_label.clone()
    } else {
        let out = "[vfinal]";
        filter.push_str(&format!(";{}{}{}", video_final_label, overlay_filters.join(","), out));
        out.to_string()
    };

    // Audio filter
    let audio_output_label = if has_any_audio {
        let af = build_audio_concat_filter(&audio_input_map, &eff_durations);
        if !af.is_empty() {
            filter.push_str(&format!(";{}", af));
            Some("[aout]".to_string())
        } else { None }
    } else { None };

    args.push("-filter_complex".into());
    args.push(filter);
    args.push("-map".into());
    args.push(output_label);
    if let Some(ref al) = audio_output_label {
        args.push("-map".into());
        args.push(al.clone());
    }

    let crf_str = quality.crf().to_string();
    args.extend(["-c:v", "libx264", "-preset", quality.preset(), "-crf", &crf_str, "-pix_fmt", "yuv420p", "-r", "30"].iter().map(|s| s.to_string()));
    if audio_output_label.is_some() {
        args.extend(["-c:a", "aac", "-b:a", "128k"].iter().map(|s| s.to_string()));
    } else {
        args.push("-an".into());
    }
    args.extend(["-shortest", "-y"].iter().map(|s| s.to_string()));
    args.push(output_path.to_string_lossy().to_string());

    let ffmpeg = ffmpeg_path();
    eprintln!("[export] Output: {:?}", output_path);

    let mut child = Command::new(&ffmpeg)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start FFmpeg export")?;

    let total_duration: f64 = eff_durations.iter().sum::<f64>()
        - (transitions.iter().filter(|t| t.transition_type != TransitionType::Cut).count() as f64 * TRANSITION_DURATION);

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

    let _ = app.emit("export-progress", 100u32);
    Ok(())
}

// ──────────────────────────────── Export with concat (Cut transitions) ────────────────────

async fn export_with_concat(
    clips: &[Clip],
    eff_durations: &[f64],
    output_path: &PathBuf,
    app: &AppHandle,
    watermark: bool,
    quality: &ExportQuality,
    clip_keystrokes: &HashMap<String, Vec<KeystrokeEvent>>,
    subtitles: &[Subtitle],
    clip_annotations: &HashMap<String, Vec<Annotation>>,
    clip_cursor_positions: &HashMap<String, Vec<CursorPosition>>,
) -> Result<()> {
    let max_w = (clips.iter().map(|c| c.region.width).max().unwrap_or(1920) / 2) * 2;
    let max_h = (clips.iter().map(|c| c.region.height).max().unwrap_or(1080) / 2) * 2;
    let has_any_audio = clips.iter().any(|c| !c.audio_paths.is_empty());
    let n = clips.len();

    let mut args: Vec<String> = Vec::new();
    for clip in clips {
        args.push("-i".into());
        args.push(clip.path.to_string_lossy().to_string());
    }

    let num_video_inputs = n;
    let mut audio_input_map: Vec<(usize, Vec<usize>)> = Vec::new();
    if has_any_audio {
        let mut input_idx = num_video_inputs;
        for (ci, clip) in clips.iter().enumerate() {
            let mut indices = Vec::new();
            for audio_path in &clip.audio_paths {
                if std::path::Path::new(audio_path).exists() {
                    args.push("-i".into());
                    args.push(audio_path.clone());
                    indices.push(input_idx);
                    input_idx += 1;
                }
            }
            audio_input_map.push((ci, indices));
        }
    }

    let mut filters = Vec::new();
    for i in 0..n {
        let clip = &clips[i];
        let cursor = clip_cursor_positions.get(&clip.id);
        let anns = clip_annotations.get(&clip.id);
        filters.push(build_clip_chain(i, clip, max_w, max_h, cursor, anns));
    }

    let inputs: String = (0..n).map(|i| format!("[s{i}]")).collect::<Vec<_>>().join("");
    let concat_label = "[vout]";
    filters.push(format!("{inputs}concat=n={n}:v=1:a=0{concat_label}"));

    // Global overlays
    let mut overlay_parts = Vec::new();
    let mut cumulative_time = 0.0;
    for (i, clip) in clips.iter().enumerate() {
        if let Some(events) = clip_keystrokes.get(&clip.id) {
            overlay_parts.extend(build_keystroke_filters(events, cumulative_time, clip.trim_start_ms));
        }
        cumulative_time += eff_durations[i];
    }
    overlay_parts.extend(build_subtitle_filters(subtitles));
    if watermark { overlay_parts.push(WATERMARK_FILTER.to_string()); }

    let video_output = if overlay_parts.is_empty() {
        concat_label.to_string()
    } else {
        let out = "[vfinal]";
        filters.push(format!("{concat_label}{}{out}", overlay_parts.join(",")));
        out.to_string()
    };

    // Audio
    let audio_output = if has_any_audio {
        let af = build_audio_concat_filter(&audio_input_map, eff_durations);
        if !af.is_empty() {
            filters.push(af);
            Some("[aout]".to_string())
        } else { None }
    } else { None };

    let filter = filters.join(";");
    args.push("-filter_complex".into());
    args.push(filter);
    args.push("-map".into());
    args.push(video_output);
    if let Some(ref al) = audio_output {
        args.push("-map".into());
        args.push(al.clone());
    }

    let crf_str = quality.crf().to_string();
    args.extend(["-c:v", "libx264", "-preset", quality.preset(), "-crf", &crf_str, "-pix_fmt", "yuv420p", "-r", "30"].iter().map(|s| s.to_string()));
    if audio_output.is_some() {
        args.extend(["-c:a", "aac", "-b:a", "128k"].iter().map(|s| s.to_string()));
    } else { args.push("-an".into()); }
    args.extend(["-shortest", "-y"].iter().map(|s| s.to_string()));
    args.push(output_path.to_string_lossy().to_string());

    let ffmpeg = ffmpeg_path();
    let output = Command::new(&ffmpeg)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run concat export")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Concat export failed: {}", stderr.chars().take(500).collect::<String>());
    }
    let _ = app.emit("export-progress", 100u32);
    Ok(())
}

// ──────────────────────────────── Export single clip ────────────────────────────────

async fn export_single_clip(
    clip: &Clip,
    output_path: &PathBuf,
    app: &AppHandle,
    watermark: bool,
    quality: &ExportQuality,
    clip_keystrokes: &HashMap<String, Vec<KeystrokeEvent>>,
    subtitles: &[Subtitle],
    clip_annotations: &HashMap<String, Vec<Annotation>>,
    clip_cursor_positions: &HashMap<String, Vec<CursorPosition>>,
) -> Result<()> {
    let _ = app.emit("export-progress", 10u32);
    let ffmpeg = ffmpeg_path();
    let has_audio = !clip.audio_paths.is_empty();

    let mut cmd_args: Vec<String> = Vec::new();

    if clip.trim_start_ms > 0 {
        cmd_args.push("-ss".into());
        cmd_args.push(format!("{:.3}", clip.trim_start_ms as f64 / 1000.0));
    }

    cmd_args.push("-i".into());
    cmd_args.push(clip.path.to_string_lossy().to_string());

    let mut audio_input_indices = Vec::new();
    let mut next_input = 1;
    for audio_path in &clip.audio_paths {
        if std::path::Path::new(audio_path).exists() {
            cmd_args.push("-i".into());
            cmd_args.push(audio_path.clone());
            audio_input_indices.push(next_input);
            next_input += 1;
        }
    }

    if clip.trim_end_ms > 0 {
        cmd_args.push("-to".into());
        let end = clip.trim_end_ms as f64 / 1000.0 - clip.trim_start_ms as f64 / 1000.0;
        cmd_args.push(format!("{:.3}", end.max(0.1)));
    }

    // Build video filter chain
    let mut vf_parts: Vec<String> = Vec::new();

    // Cursor zoom
    if let Some(positions) = clip_cursor_positions.get(&clip.id) {
        if let Some(zoom) = build_cursor_zoom_filter(positions, clip.trim_start_ms, clip.region.width, clip.region.height) {
            vf_parts.push(zoom);
        }
    }

    // Annotations
    if let Some(anns) = clip_annotations.get(&clip.id) {
        vf_parts.extend(build_annotation_draw_filters(anns, clip.region.width, clip.region.height));
    }

    // Keystrokes
    if let Some(events) = clip_keystrokes.get(&clip.id) {
        vf_parts.extend(build_keystroke_filters(events, 0.0, clip.trim_start_ms));
    }

    // Subtitles
    vf_parts.extend(build_subtitle_filters(subtitles));

    // Watermark
    if watermark {
        vf_parts.push(WATERMARK_FILTER.to_string());
    }

    let need_filter_complex = !vf_parts.is_empty() || audio_input_indices.len() > 1;

    if need_filter_complex {
        let mut fc_parts = Vec::new();

        let video_chain = if vf_parts.is_empty() {
            "[0:v]null[vout]".to_string()
        } else {
            format!("[0:v]{}[vout]", vf_parts.join(","))
        };
        fc_parts.push(video_chain);

        if audio_input_indices.len() > 1 {
            let mix: String = audio_input_indices.iter().map(|i| format!("[{}:a]", i)).collect();
            fc_parts.push(format!("{mix}amix=inputs={}:duration=first[aout]", audio_input_indices.len()));
        }

        cmd_args.push("-filter_complex".into());
        cmd_args.push(fc_parts.join(";"));
        cmd_args.push("-map".into());
        cmd_args.push("[vout]".into());

        if audio_input_indices.len() > 1 {
            cmd_args.push("-map".into());
            cmd_args.push("[aout]".into());
        } else if audio_input_indices.len() == 1 {
            cmd_args.push("-map".into());
            cmd_args.push(format!("{}:a", audio_input_indices[0]));
        }
    } else if audio_input_indices.len() == 1 {
        cmd_args.push("-map".into());
        cmd_args.push("0:v".into());
        cmd_args.push("-map".into());
        cmd_args.push(format!("{}:a", audio_input_indices[0]));
    }

    let crf_str = quality.crf().to_string();
    cmd_args.extend(["-c:v", "libx264", "-preset", quality.preset(), "-crf", &crf_str, "-pix_fmt", "yuv420p", "-r", "30"].iter().map(|s| s.to_string()));
    if has_audio {
        cmd_args.extend(["-c:a", "aac", "-b:a", "128k"].iter().map(|s| s.to_string()));
    } else { cmd_args.push("-an".into()); }
    cmd_args.extend(["-shortest", "-y"].iter().map(|s| s.to_string()));
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
        anyhow::bail!("FFmpeg exited with code {:?}. Stderr: {}", output.status.code(), stderr.chars().take(500).collect::<String>());
    }

    let _ = app.emit("export-progress", 100u32);
    Ok(())
}

// ──────────────────────────────── Export GIF ────────────────────────────────

pub async fn export_gif(
    clips: &[Clip],
    transitions: &[Transition],
    output_path: &PathBuf,
    app: &AppHandle,
    watermark: bool,
    quality: &ExportQuality,
    clip_keystrokes: &HashMap<String, Vec<KeystrokeEvent>>,
    subtitles: &[Subtitle],
    clip_annotations: &HashMap<String, Vec<Annotation>>,
    clip_cursor_positions: &HashMap<String, Vec<CursorPosition>>,
) -> Result<()> {
    if clips.is_empty() {
        anyhow::bail!("No clips to export as GIF");
    }
    for (i, clip) in clips.iter().enumerate() {
        if !clip.path.exists() {
            anyhow::bail!("Clip {} file not found: {:?}", i + 1, clip.path);
        }
    }

    let _ = app.emit("export-progress", 5u32);
    let temp_mp4 = output_path.with_extension("tmp.mp4");
    let temp_quality = ExportQuality::Low;

    if clips.len() == 1 {
        export_single_clip(&clips[0], &temp_mp4, app, watermark, &temp_quality, clip_keystrokes, subtitles, clip_annotations, clip_cursor_positions).await?;
    } else {
        export_mp4(clips, transitions, &temp_mp4, app, watermark, &temp_quality, clip_keystrokes, subtitles, clip_annotations, clip_cursor_positions).await?;
    }
    let _ = app.emit("export-progress", 50u32);

    let fps = match quality { ExportQuality::High => 15, ExportQuality::Medium => 12, ExportQuality::Low => 8 };
    let max_width = match quality { ExportQuality::High => 640, ExportQuality::Medium => 480, ExportQuality::Low => 320 };

    let palette_path = output_path.with_extension("palette.png");
    let ffmpeg = ffmpeg_path();

    // Pass 1: palette
    let pf = format!("fps={fps},scale={max_width}:-1:flags=lanczos,palettegen=stats_mode=diff");
    let output = Command::new(&ffmpeg)
        .args(["-i", &temp_mp4.to_string_lossy(), "-vf", &pf, "-y", &palette_path.to_string_lossy()])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped())
        .output().await.context("Failed to generate GIF palette")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Palette generation failed: {}", stderr.chars().take(500).collect::<String>());
    }
    let _ = app.emit("export-progress", 75u32);

    // Pass 2: GIF
    let gf = format!("fps={fps},scale={max_width}:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5");
    let output = Command::new(&ffmpeg)
        .args(["-i", &temp_mp4.to_string_lossy(), "-i", &palette_path.to_string_lossy(), "-filter_complex", &gf, "-y", &output_path.to_string_lossy()])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped())
        .output().await.context("Failed to generate GIF")?;

    let _ = std::fs::remove_file(&temp_mp4);
    let _ = std::fs::remove_file(&palette_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("GIF generation failed: {}", stderr.chars().take(500).collect::<String>());
    }
    let _ = app.emit("export-progress", 100u32);
    Ok(())
}

// ──────────────────────────────── Preview ────────────────────────────────

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
        durations.push(probe_duration(&clip.path).await?);
    }
    let eff_durations: Vec<f64> = clips.iter().zip(durations.iter())
        .map(|(c, d)| effective_duration(c, *d)).collect();

    let max_w = clips.iter().map(|c| c.region.width).max().unwrap_or(1920);
    let max_h = clips.iter().map(|c| c.region.height).max().unwrap_or(1080);
    let prev_w = ((max_w / 2) / 2 * 2).max(320);
    let prev_h = ((max_h / 2) / 2 * 2).max(240);

    // Preview without annotations/cursor/keystrokes for speed
    let empty_ann: HashMap<String, Vec<Annotation>> = HashMap::new();
    let empty_cur: HashMap<String, Vec<CursorPosition>> = HashMap::new();

    let mut args: Vec<String> = Vec::new();
    for clip in clips {
        args.push("-i".into());
        args.push(clip.path.to_string_lossy().to_string());
    }

    let filter = build_filter_complex_with_trim(
        clips, &eff_durations, transitions, prev_w, prev_h,
        &empty_ann, &empty_cur,
    );
    args.push("-filter_complex".into());
    args.push(filter);

    let final_label = if clips.len() == 2 { "[v0]".to_string() } else { format!("[v{}]", clips.len() - 2) };
    args.push("-map".into());
    args.push(final_label);
    args.extend(["-c:v", "libx264", "-preset", "ultrafast", "-crf", "30", "-pix_fmt", "yuv420p", "-r", "24", "-an", "-y"].iter().map(|s| s.to_string()));
    args.push(output_path.to_string_lossy().to_string());

    let ffmpeg = ffmpeg_path();
    let mut child = Command::new(&ffmpeg)
        .args(&args)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped())
        .spawn().context("Failed to start FFmpeg preview")?;

    let total_duration: f64 = eff_durations.iter().sum::<f64>()
        - (transitions.iter().filter(|t| t.transition_type != TransitionType::Cut).count() as f64 * TRANSITION_DURATION);

    let mut stderr_log = String::new();
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
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

async fn preview_single_clip(clip: &Clip, output_path: &PathBuf, app: &AppHandle) -> Result<()> {
    let _ = app.emit("preview-progress", 10u32);
    let ffmpeg = ffmpeg_path();
    let prev_w = ((clip.region.width / 2) / 2 * 2).max(320);
    let prev_h = ((clip.region.height / 2) / 2 * 2).max(240);

    let mut cmd_args: Vec<String> = Vec::new();
    if clip.trim_start_ms > 0 {
        cmd_args.push("-ss".into());
        cmd_args.push(format!("{:.3}", clip.trim_start_ms as f64 / 1000.0));
    }
    cmd_args.push("-i".into());
    cmd_args.push(clip.path.to_string_lossy().to_string());
    if clip.trim_end_ms > 0 {
        cmd_args.push("-to".into());
        let end = clip.trim_end_ms as f64 / 1000.0 - clip.trim_start_ms as f64 / 1000.0;
        cmd_args.push(format!("{:.3}", end.max(0.1)));
    }
    let scale = format!("scale={}:{}", prev_w, prev_h);
    cmd_args.extend(["-vf", &scale, "-c:v", "libx264", "-preset", "ultrafast", "-crf", "30", "-pix_fmt", "yuv420p", "-r", "24", "-an", "-y"].iter().map(|s| s.to_string()));
    cmd_args.push(output_path.to_string_lossy().to_string());

    let output = Command::new(&ffmpeg)
        .args(&cmd_args)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped())
        .output().await.context("Failed to preview single clip")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Preview failed: {}", stderr.chars().take(500).collect::<String>());
    }
    let _ = app.emit("preview-progress", 100u32);
    Ok(())
}

// ──────────────────────────────── Utils ────────────────────────────────

fn extract_time(line: &str) -> Option<f64> {
    let idx = line.find("time=")?;
    let time_str = &line[idx + 5..];
    let end = time_str.find(' ').unwrap_or(time_str.len());
    let time_str = &time_str[..end];
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().ok()?;
        let m: f64 = parts[1].parse().ok()?;
        let s: f64 = parts[2].parse().ok()?;
        Some(h * 3600.0 + m * 60.0 + s)
    } else { None }
}

// ──────────────────────────────── Tests ────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Region;

    fn make_clip(trim_start: u64, trim_end: u64) -> Clip {
        Clip {
            id: "test".into(),
            path: PathBuf::from("test.mp4"),
            duration_ms: 10000,
            region: Region { x: 0, y: 0, width: 1920, height: 1080 },
            has_audio: false,
            thumbnail_path: None,
            trim_start_ms: trim_start,
            trim_end_ms: trim_end,
            audio_paths: vec![],
        }
    }

    fn make_subtitle(text: &str, start: u64, end: u64, pos: SubtitlePosition, font_size: u32, color: &str) -> Subtitle {
        Subtitle {
            id: "sub1".into(),
            text: text.into(),
            start_ms: start,
            end_ms: end,
            position: pos,
            font_size,
            color: color.into(),
        }
    }

    // ── escape_drawtext ──

    #[test]
    fn test_escape_drawtext_plain() {
        assert_eq!(escape_drawtext("Hello world"), "Hello world");
    }

    #[test]
    fn test_escape_drawtext_backslash() {
        assert_eq!(escape_drawtext("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_drawtext_colon() {
        assert_eq!(escape_drawtext("time:12:30"), "time\\:12\\:30");
    }

    #[test]
    fn test_escape_drawtext_percent() {
        assert_eq!(escape_drawtext("100%"), "100%%");
    }

    #[test]
    fn test_escape_drawtext_combo() {
        let result = escape_drawtext("a\\b:c%d");
        assert!(result.contains("\\\\"));
        assert!(result.contains("\\:"));
        assert!(result.contains("%%"));
    }

    // ── build_piecewise_lerp ──

    #[test]
    fn test_lerp_empty() {
        assert_eq!(build_piecewise_lerp(&[]), "0");
    }

    #[test]
    fn test_lerp_single() {
        let result = build_piecewise_lerp(&[(0.0, 0.5)]);
        assert_eq!(result, "0.5000");
    }

    #[test]
    fn test_lerp_two_keyframes() {
        let result = build_piecewise_lerp(&[(0.0, 0.0), (1.0, 1.0)]);
        assert!(result.contains("if(lt(t"));
    }

    #[test]
    fn test_lerp_three_keyframes() {
        let result = build_piecewise_lerp(&[(0.0, 0.0), (1.0, 0.5), (2.0, 1.0)]);
        // Should have nested if expressions
        let if_count = result.matches("if(lt(t").count();
        assert_eq!(if_count, 2);
    }

    // ── annotation_color_ffmpeg ──

    #[test]
    fn test_color_hex() {
        assert_eq!(annotation_color_ffmpeg("#ff0000"), "0xff0000");
    }

    #[test]
    fn test_color_name() {
        assert_eq!(annotation_color_ffmpeg("white"), "white");
    }

    #[test]
    fn test_color_hex_black() {
        assert_eq!(annotation_color_ffmpeg("#000000"), "0x000000");
    }

    // ── effective_duration ──

    #[test]
    fn test_duration_no_trim() {
        let clip = make_clip(0, 0);
        assert!((effective_duration(&clip, 5.0) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_duration_trim_start() {
        let clip = make_clip(1000, 0);
        assert!((effective_duration(&clip, 5.0) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_duration_trim_end() {
        let clip = make_clip(0, 3000);
        assert!((effective_duration(&clip, 5.0) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_duration_trim_both() {
        let clip = make_clip(1000, 4000);
        assert!((effective_duration(&clip, 5.0) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_duration_min_clamp() {
        let clip = make_clip(5000, 5000);
        assert!(effective_duration(&clip, 5.0) >= 0.1);
    }

    // ── xfade_name ──

    #[test]
    fn test_xfade_fade() {
        assert_eq!(xfade_name(&TransitionType::Fade), "fade");
    }

    #[test]
    fn test_xfade_zoom() {
        assert_eq!(xfade_name(&TransitionType::Zoom), "zoomin");
    }

    #[test]
    fn test_xfade_slide() {
        assert_eq!(xfade_name(&TransitionType::Slide), "slideleft");
    }

    #[test]
    fn test_xfade_cut_maps_to_fade() {
        assert_eq!(xfade_name(&TransitionType::Cut), "fade");
    }

    // ── all_cuts ──

    #[test]
    fn test_all_cuts_true() {
        let t = vec![Transition { transition_type: TransitionType::Cut }];
        assert!(all_cuts(&t));
    }

    #[test]
    fn test_all_cuts_false() {
        let t = vec![
            Transition { transition_type: TransitionType::Cut },
            Transition { transition_type: TransitionType::Fade },
        ];
        assert!(!all_cuts(&t));
    }

    #[test]
    fn test_all_cuts_empty() {
        assert!(all_cuts(&[]));
    }

    // ── build_subtitle_filters ──

    #[test]
    fn test_subtitle_basic() {
        let subs = vec![make_subtitle("Hello", 0, 3000, SubtitlePosition::Bottom, 32, "#ffffff")];
        let filters = build_subtitle_filters(&subs);
        assert_eq!(filters.len(), 1);
        assert!(filters[0].contains("drawtext="));
        assert!(filters[0].contains("Hello"));
        assert!(filters[0].contains("fontsize=32"));
    }

    #[test]
    fn test_subtitle_color() {
        let subs = vec![make_subtitle("Red", 0, 1000, SubtitlePosition::Bottom, 32, "#ff0000")];
        let filters = build_subtitle_filters(&subs);
        assert!(filters[0].contains("fontcolor=0xff0000"));
    }

    #[test]
    fn test_subtitle_position_top() {
        let subs = vec![make_subtitle("Top", 0, 1000, SubtitlePosition::Top, 32, "#ffffff")];
        let filters = build_subtitle_filters(&subs);
        assert!(filters[0].contains("y=30"));
    }

    #[test]
    fn test_subtitle_position_center() {
        let subs = vec![make_subtitle("Center", 0, 1000, SubtitlePosition::Center, 32, "#ffffff")];
        let filters = build_subtitle_filters(&subs);
        assert!(filters[0].contains("y=(h-th)/2"));
    }

    #[test]
    fn test_subtitle_position_bottom() {
        let subs = vec![make_subtitle("Bottom", 0, 1000, SubtitlePosition::Bottom, 32, "#ffffff")];
        let filters = build_subtitle_filters(&subs);
        assert!(filters[0].contains("y=h-th-30"));
    }

    #[test]
    fn test_subtitle_custom_fontsize() {
        let subs = vec![make_subtitle("Big", 0, 1000, SubtitlePosition::Bottom, 56, "#ffffff")];
        let filters = build_subtitle_filters(&subs);
        assert!(filters[0].contains("fontsize=56"));
    }

    #[test]
    fn test_subtitle_default_fontsize() {
        let subs = vec![make_subtitle("Default", 0, 1000, SubtitlePosition::Bottom, 0, "#ffffff")];
        let filters = build_subtitle_filters(&subs);
        assert!(filters[0].contains("fontsize=32"));
    }

    #[test]
    fn test_subtitle_empty() {
        assert!(build_subtitle_filters(&[]).is_empty());
    }

    // ── build_cursor_zoom_filter ──

    #[test]
    fn test_cursor_zoom_empty() {
        assert!(build_cursor_zoom_filter(&[], 0, 1920, 1080).is_none());
    }

    #[test]
    fn test_cursor_zoom_stationary() {
        // Cursor barely moves (<10% in X and Y)
        let positions = vec![
            CursorPosition { timestamp_ms: 0, x: 0.50, y: 0.50 },
            CursorPosition { timestamp_ms: 500, x: 0.52, y: 0.51 },
            CursorPosition { timestamp_ms: 1000, x: 0.51, y: 0.52 },
        ];
        assert!(build_cursor_zoom_filter(&positions, 0, 1920, 1080).is_none());
    }

    #[test]
    fn test_cursor_zoom_moving() {
        // Cursor moves >10% in X
        let positions = vec![
            CursorPosition { timestamp_ms: 0, x: 0.1, y: 0.5 },
            CursorPosition { timestamp_ms: 1000, x: 0.5, y: 0.5 },
            CursorPosition { timestamp_ms: 2000, x: 0.9, y: 0.5 },
        ];
        let result = build_cursor_zoom_filter(&positions, 0, 1920, 1080);
        assert!(result.is_some());
        let filter = result.unwrap();
        assert!(filter.contains("crop="));
        assert!(filter.contains("scale=1920:1080"));
    }

    #[test]
    fn test_cursor_zoom_with_trim_offset() {
        // Positions all in same area — with trim, keyframes shift but motion check uses raw positions
        let positions = vec![
            CursorPosition { timestamp_ms: 2000, x: 0.5, y: 0.5 },
            CursorPosition { timestamp_ms: 3000, x: 0.9, y: 0.5 },
            CursorPosition { timestamp_ms: 4000, x: 0.5, y: 0.5 },
        ];
        // Movement > 10%, should produce a filter. Trim offsets keyframe times.
        let result = build_cursor_zoom_filter(&positions, 1000, 1920, 1080);
        assert!(result.is_some());
    }

    // ── extract_time ──

    #[test]
    fn test_extract_time_valid() {
        let line = "frame=  100 fps=30 time=00:01:23.45 bitrate=1234kbits/s";
        let t = extract_time(line).unwrap();
        assert!((t - 83.45).abs() < 0.01);
    }

    #[test]
    fn test_extract_time_zero() {
        let line = "time=00:00:00.00 something";
        assert!((extract_time(line).unwrap()).abs() < 0.01);
    }

    #[test]
    fn test_extract_time_no_match() {
        assert!(extract_time("no time here").is_none());
    }

    // ── build_keystroke_filters ──

    #[test]
    fn test_keystroke_filter_basic() {
        let events = vec![KeystrokeEvent { timestamp_ms: 1000, key_name: "A".into() }];
        let filters = build_keystroke_filters(&events, 0.0, 0);
        assert_eq!(filters.len(), 1);
        assert!(filters[0].contains("drawtext="));
        assert!(filters[0].contains("A"));
    }

    #[test]
    fn test_keystroke_filter_time_offset() {
        let events = vec![KeystrokeEvent { timestamp_ms: 1000, key_name: "B".into() }];
        let filters = build_keystroke_filters(&events, 5.0, 0);
        assert_eq!(filters.len(), 1);
        // Event at 1.0s + offset 5.0s = 6.0s
        assert!(filters[0].contains("6.0"));
    }

    #[test]
    fn test_keystroke_filter_trim_offset() {
        let events = vec![KeystrokeEvent { timestamp_ms: 3000, key_name: "C".into() }];
        let filters = build_keystroke_filters(&events, 0.0, 2000);
        assert_eq!(filters.len(), 1);
        // Event at 3.0s - trim 2.0s = 1.0s
        assert!(filters[0].contains("1.0"));
    }

    #[test]
    fn test_keystroke_filter_skips_negative_time() {
        let events = vec![KeystrokeEvent { timestamp_ms: 500, key_name: "D".into() }];
        let filters = build_keystroke_filters(&events, 0.0, 2000);
        // Event at 0.5s - trim 2.0s = -1.5s → skipped
        assert!(filters.is_empty());
    }
}
