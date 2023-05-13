use actix_files::NamedFile;
use actix_web::web::Bytes;
use std::{fs, path::PathBuf};

fn get_artifact_path(artifact_id: String, team_id: String) -> PathBuf {
    // env temp dir
    let temp_dir = std::env::temp_dir();
    let artifact_path = format!("{}/{}/{}", temp_dir.to_str().unwrap(), team_id, artifact_id);
    // check if artifact exists
    PathBuf::from(artifact_path)
}

pub async fn get_cached_artifact(
    artifact_id: String,
    team_id: String,
) -> Result<NamedFile, String> {
    let path = get_artifact_path(artifact_id, team_id);
    if !path.exists() {
        return Err(format!(
            "Artifact {} doesn't exist.",
            path.to_str().unwrap()
        ));
    }
    let file: NamedFile = NamedFile::open_async(path).await.unwrap();
    Ok(file)
}

pub fn exists_cached_artifact(artifact_id: String, team_id: String) -> Result<bool, String> {
    let artifact_path = get_artifact_path(artifact_id, team_id);
    if !artifact_path.exists() {
        return Err(format!(
            "Artifact {} doesn't exist.",
            artifact_path.to_str().unwrap()
        ));
    }
    Ok(true)
}

pub fn create_cached_artifact(
    artifact_id: String,
    team_id: String,
    artifact: Bytes,
) -> Result<String, String> {
    // create artifact path
    let artifact_path = get_artifact_path(artifact_id, team_id);
    // create artifact dir
    let artifact_dir = artifact_path.parent().unwrap();
    fs::create_dir_all(artifact_dir).expect("Failed to create artifact dir.");
    // write artifact
    fs::write(artifact_path.clone(), artifact).expect("Failed to write artifact.");
    Ok(artifact_path.to_str().unwrap().to_string())
}
