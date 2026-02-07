use crate::types::*;
use std::collections::HashMap;
use std::path::PathBuf;

fn projects_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_local_dir()
        .ok_or("Cannot find local data directory")?
        .join("ClipFlow")
        .join("projects");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn save_project(
    existing_id: Option<String>,
    name: &str,
    clips: &[Clip],
    transitions: &[Transition],
    audio_source: AudioSource,
    annotations: &HashMap<String, Vec<Annotation>>,
    subtitles: &[Subtitle],
) -> Result<String, String> {
    let now = chrono::Local::now().to_rfc3339();
    let project_id = existing_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let project = Project {
        id: project_id.clone(),
        name: name.to_string(),
        created_at: now.clone(),
        updated_at: now,
        clips: clips.to_vec(),
        transitions: transitions.to_vec(),
        settings: ProjectSettings {
            audio_source,
            watermark_enabled: true,
            export_format: ExportFormat::Mp4,
            export_quality: ExportQuality::Medium,
        },
        annotations: annotations.clone(),
        subtitles: subtitles.to_vec(),
    };

    let dir = projects_dir()?;
    let file_path = dir.join(format!("{}.json", project_id));
    let json = serde_json::to_string_pretty(&project).map_err(|e| e.to_string())?;
    std::fs::write(&file_path, json).map_err(|e| e.to_string())?;

    Ok(project_id)
}

pub fn load_project(project_id: &str) -> Result<Project, String> {
    let dir = projects_dir()?;
    let file_path = dir.join(format!("{}.json", project_id));

    if !file_path.exists() {
        return Err(format!("Project not found: {}", project_id));
    }

    let json = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let project: Project = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(project)
}

pub fn list_projects() -> Result<Vec<ProjectSummary>, String> {
    let dir = projects_dir()?;
    let mut summaries = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(project) = serde_json::from_str::<Project>(&json) {
                        let total_duration_ms: u64 = project.clips.iter()
                            .map(|c| c.duration_ms)
                            .sum();
                        summaries.push(ProjectSummary {
                            id: project.id,
                            name: project.name,
                            created_at: project.created_at,
                            updated_at: project.updated_at,
                            clip_count: project.clips.len(),
                            total_duration_ms,
                        });
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(summaries)
}

pub fn delete_project(project_id: &str) -> Result<(), String> {
    let dir = projects_dir()?;
    let file_path = dir.join(format!("{}.json", project_id));

    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
