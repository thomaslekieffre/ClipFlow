use crate::types::Region;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Child;

/// Clamp a region so it stays within the virtual desktop bounds for gdigrab.
/// Negative coordinates (from window shadow borders) and overflow are trimmed.
/// Width/height are forced to even values (required by libx264 yuv420p).
fn clamp_region(region: &Region) -> Region {
    let mut x = region.x;
    let mut y = region.y;
    let mut w = region.width as i32;
    let mut h = region.height as i32;

    // If origin is negative, shrink dimensions and clamp to 0
    if x < 0 {
        w += x; // reduce width by the overshoot
        x = 0;
    }
    if y < 0 {
        h += y;
        y = 0;
    }

    // Ensure positive dimensions
    if w < 2 { w = 2; }
    if h < 2 { h = 2; }

    // Force even dimensions for h264 yuv420p
    let w = (w as u32) & !1;
    let h = (h as u32) & !1;

    Region { x, y, width: w, height: h }
}

/// Start screen capture of a region using FFmpeg's gdigrab
pub fn start_capture(
    region: &Region,
    output_path: &PathBuf,
    framerate: u32,
) -> Result<Child> {
    let region = clamp_region(region);

    let child = crate::ffmpeg_command()
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
    let child = crate::ffmpeg_command()
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
    // Drain stderr concurrently to avoid pipe buffer deadlock
    let stderr_task = child.stderr.take().map(|mut stderr| {
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::new();
            let _ = stderr.read_to_end(&mut buf).await;
            String::from_utf8_lossy(&buf).to_string()
        })
    });

    // Send quit command
    if let Some(stdin) = child.stdin.as_mut() {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(b"q").await;
    }
    // Drop stdin so FFmpeg sees EOF if 'q' isn't enough
    drop(child.stdin.take());

    let status = child.wait().await?;

    let stderr_output = match stderr_task {
        Some(handle) => handle.await.unwrap_or_default(),
        None => String::new(),
    };

    if !status.success() {
        let snippet: String = stderr_output.chars().rev().take(500).collect::<String>().chars().rev().collect();
        eprintln!("[screen] FFmpeg capture exited with {:?}. Stderr: {}", status.code(), snippet);
        anyhow::bail!("FFmpeg capture failed (exit code {:?}): {}", status.code(), snippet);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_region_normal() {
        let r = clamp_region(&Region { x: 100, y: 200, width: 800, height: 600 });
        assert_eq!(r, Region { x: 100, y: 200, width: 800, height: 600 });
    }

    #[test]
    fn test_clamp_region_negative_y() {
        // Window shadow: y=-8, height=1048 → y=0, height=1040
        let r = clamp_region(&Region { x: 1912, y: -8, width: 1936, height: 1048 });
        assert_eq!(r.x, 0.max(1912));
        assert_eq!(r.y, 0);
        assert_eq!(r.height, 1040);
    }

    #[test]
    fn test_clamp_region_negative_x_and_y() {
        let r = clamp_region(&Region { x: -10, y: -8, width: 1000, height: 800 });
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
        assert_eq!(r.width, 990);
        assert_eq!(r.height, 792);
    }

    #[test]
    fn test_clamp_region_odd_dimensions() {
        let r = clamp_region(&Region { x: 0, y: 0, width: 801, height: 601 });
        assert_eq!(r.width, 800);
        assert_eq!(r.height, 600);
    }

    #[test]
    fn test_clamp_region_minimum_size() {
        // Extremely negative: width would go to 0 or negative
        let r = clamp_region(&Region { x: -500, y: -500, width: 100, height: 100 });
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
        assert!(r.width >= 2);
        assert!(r.height >= 2);
    }
}

/// Generate a thumbnail from a video file (first frame)
pub fn generate_thumbnail(video_path: &PathBuf, thumbnail_path: &PathBuf) -> Result<()> {
    crate::ffmpeg_command_sync()
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
