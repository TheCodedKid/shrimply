use std::fs;
use std::path::Path;

use shrimply_project::project::{self, Project};

pub fn export(project: &Project, path: &Path) -> Result<(), String> {
    let data = project::serialize_project_json(path, project)
        .map_err(|error| format!("Could not serialize project: {error}"))?;
    fs::write(path, data).map_err(|error| format!("Could not save file: {error}"))
}
