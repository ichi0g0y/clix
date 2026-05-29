use tempfile::TempDir;

use super::{ProjectsConfig, load, pick_project, save};
use crate::test_support::EnvGuard;

#[test]
fn config_round_trip_preserves_default_and_mappings() {
    let dir = TempDir::new().unwrap();
    let _env = EnvGuard::set_xdg(dir.path());

    let mut cfg = ProjectsConfig {
        default: Some("default-project".to_string()),
        ..ProjectsConfig::default()
    };
    cfg.mappings
        .insert("acme".to_string(), "acme-project".to_string());

    save(&cfg).unwrap();
    let loaded = load().unwrap();

    assert_eq!(loaded.default.as_deref(), Some("default-project"));
    assert_eq!(loaded.mappings["acme"], "acme-project");
}

#[test]
fn missing_config_returns_default() {
    let dir = TempDir::new().unwrap();
    let _env = EnvGuard::set_xdg(dir.path());

    let cfg = load().unwrap();

    assert!(cfg.default.is_none());
    assert!(cfg.mappings.is_empty());
}

#[test]
fn pick_project_prefers_mapping_then_default_source() {
    let mut cfg = ProjectsConfig {
        default: Some("default-project".to_string()),
        ..ProjectsConfig::default()
    };
    cfg.mappings
        .insert("acme".to_string(), "acme-project".to_string());

    assert_eq!(
        pick_project(&cfg, "acme", true).as_deref(),
        Some("acme-project")
    );
    assert_eq!(
        pick_project(&cfg, "other", true).as_deref(),
        Some("default-project")
    );
    assert_eq!(pick_project(&cfg, "other", false), None);
}
