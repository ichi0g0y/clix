use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub mappings: BTreeMap<String, String>,
}

pub fn config_dir() -> Result<PathBuf, Error> {
    let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else {
        dirs::home_dir()
            .ok_or(Error::ConfigDirUnavailable)?
            .join(".config")
    };
    Ok(base.join("stripex"))
}

pub fn config_path() -> Result<PathBuf, Error> {
    Ok(config_dir()?.join("projects.yml"))
}

pub fn load() -> Result<ProjectsConfig, Error> {
    let path = config_path()?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ProjectsConfig::default());
        }
        Err(e) => {
            return Err(Error::ConfigParseError {
                path,
                msg: e.to_string(),
            });
        }
    };

    serde_yml::from_str(&content).map_err(|e| Error::ConfigParseError {
        path,
        msg: e.to_string(),
    })
}

pub fn save(cfg: &ProjectsConfig) -> Result<(), Error> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::ConfigWriteError {
            path: parent.to_path_buf(),
            msg: e.to_string(),
        })?;
    }

    let yaml = serde_yml::to_string(cfg).map_err(|e| Error::ConfigWriteError {
        path: path.clone(),
        msg: e.to_string(),
    })?;
    fs::write(&path, yaml).map_err(|e| Error::ConfigWriteError {
        path,
        msg: e.to_string(),
    })
}

pub fn pick_project(
    cfg: &ProjectsConfig,
    trigger: &str,
    is_default_source: bool,
) -> Option<String> {
    if let Some(project) = cfg.mappings.get(trigger) {
        if !project.is_empty() {
            return Some(project.clone());
        }
    }

    if is_default_source {
        return cfg.default.clone();
    }

    None
}

#[cfg(test)]
mod tests;
