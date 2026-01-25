use crate::types::Region;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Find the FFmpeg binary path (auto-downloaded by ffmpeg-sidecar)
fn ffmpeg_path() -> PathBuf {
    ffmpeg_sidecar::paths::ffmpeg_path()
}

/// Start screen capture of a region using FFmpeg's gdigrab
pub fn start_capture(
    region: &Region,
    output_path: &PathBuf,
    framerate: u32,
) -> Result<Child> {
    let child = Command::new(ffmpeg_path())
        .args([
            "-f", "gdigrab",
            "-framerate", &framerate.to_string(),
            "-offset_x", &region.x.to_string(),
            "-offset_y", &region.y.to_string(),
            "-video_size", &format!("{}x{}", region.width, region.height),
            "-draw_mouse", "1",
            "-i", "desktop",
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-y",
            &output_path.to_string_lossy(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    Ok(child)
}

/// Start full-screen capture using FFmpeg's gdigrab
pub fn start_fullscreen_capture(
    output_path: &PathBuf,
    framerate: u32,
) -> Result<Child> {
    let child = Command::new(ffmpeg_path())
        .args([
            "-f", "gdigrab",
            "-framerate", &framerate.to_string(),
            "-draw_mouse", "1",
            "-i", "desktop",
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-y",
            &output_path.to_string_lossy(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    Ok(child)
}

/// Stop an FFmpeg capture by sending 'q' to stdin
pub async fn stop_capture(child: &mut Child) -> Result<()> {
    if let Some(stdin) = child.stdin.as_mut() {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(b"q").await;
    }
    child.wait().await?;
    Ok(())
}

/// Generate a thumbnail from a video file (first frame)
pub fn generate_thumbnail(video_path: &PathBuf, thumbnail_path: &PathBuf) -> Result<()> {
    std::process::Command::new(ffmpeg_path())
        .args([
            "-i", &video_path.to_string_lossy(),
            "-vframes", "1",
            "-vf", "scale=192:-1",
            "-y",
            &thumbnail_path.to_string_lossy(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;
    Ok(())
}
