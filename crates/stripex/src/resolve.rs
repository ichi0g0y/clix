use std::env;

use clix_core::git;

use crate::args::ParsedArgs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSource {
    ExplicitProject,
    EnvApiKey,
    GitRemote,
    Default,
}

pub fn resolve_trigger(_parsed: &ParsedArgs) -> (String, TriggerSource) {
    match git::get_remote_owner() {
        Ok(owner) => (owner, TriggerSource::GitRemote),
        Err(_) => (String::new(), TriggerSource::Default),
    }
}

pub fn has_stripe_env_key() -> bool {
    stripe_env_key().is_some()
}

pub fn stripe_env_key() -> Option<(&'static str, String)> {
    env_value("STRIPE_API_KEY")
        .map(|value| ("STRIPE_API_KEY", value))
        .or_else(|| env_value("STRIPE_SECRET_KEY").map(|value| ("STRIPE_SECRET_KEY", value)))
}

pub fn resolve_project_name(
    trigger: &str,
    source: TriggerSource,
) -> Result<Option<String>, crate::error::Error> {
    let cfg = crate::config::load()?;
    Ok(crate::config::pick_project(
        &cfg,
        trigger,
        source == TriggerSource::Default,
    ))
}

pub fn print_dry_run(parsed: &ParsedArgs) -> Result<(), crate::error::Error> {
    eprintln!("stripex dry-run:");

    if let Some(project) = parsed.explicit_project.as_deref() {
        eprintln!("  action: pass through unchanged (you passed -p {project})");
        return Ok(());
    }

    if has_stripe_env_key() || parsed.api_key_present {
        if let Some((name, _)) = stripe_env_key() {
            eprintln!("  action: pass through unchanged (explicit API key via env:{name})");
        } else {
            eprintln!("  action: pass through unchanged (explicit API key via --api-key)");
        }
        return Ok(());
    }

    let (trigger, source) = resolve_trigger(parsed);
    let project = resolve_project_name(&trigger, source)?;

    eprintln!("  trigger: {}", trigger_label(&trigger));
    eprintln!("  trigger source: {}", source_label(source));
    if let Some(project) = project {
        eprintln!("  action: inject -p {project}");
    } else {
        eprintln!("  action: no mapping/default -> stripe uses its own [default] project");
    }
    Ok(())
}

fn env_value(name: &'static str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn trigger_label(trigger: &str) -> &str {
    if trigger.is_empty() {
        "(default)"
    } else {
        trigger
    }
}

fn source_label(source: TriggerSource) -> &'static str {
    match source {
        TriggerSource::ExplicitProject => "explicit project",
        TriggerSource::EnvApiKey => "env api key",
        TriggerSource::GitRemote => "git remote",
        TriggerSource::Default => "default",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::test_support::EnvGuard;

    #[test]
    fn stripe_api_key_env_is_detected() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());

        unsafe {
            env::set_var("STRIPE_API_KEY", "sk_test");
        }

        assert!(has_stripe_env_key());
        assert_eq!(
            stripe_env_key(),
            Some(("STRIPE_API_KEY", "sk_test".to_string()))
        );
    }

    #[test]
    fn stripe_secret_key_env_is_detected() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());

        unsafe {
            env::set_var("STRIPE_SECRET_KEY", "sk_secret");
        }

        assert!(has_stripe_env_key());
        assert_eq!(
            stripe_env_key(),
            Some(("STRIPE_SECRET_KEY", "sk_secret".to_string()))
        );
    }

    #[test]
    fn stripe_api_key_env_is_preferred() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());

        unsafe {
            env::set_var("STRIPE_API_KEY", "sk_api");
            env::set_var("STRIPE_SECRET_KEY", "sk_secret");
        }

        assert_eq!(
            stripe_env_key(),
            Some(("STRIPE_API_KEY", "sk_api".to_string()))
        );
    }

    #[test]
    fn empty_or_missing_env_keys_are_ignored() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());

        unsafe {
            env::set_var("STRIPE_API_KEY", "");
            env::remove_var("STRIPE_SECRET_KEY");
        }

        assert!(!has_stripe_env_key());
        assert_eq!(stripe_env_key(), None);
    }

    #[test]
    fn resolve_project_name_returns_mapping_hit() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());
        write_projects_yml(
            xdg.path(),
            r#"
default: fallback-project
mappings:
  acme: acme-project
"#,
        );

        let resolved = resolve_project_name("acme", TriggerSource::GitRemote).unwrap();

        assert_eq!(resolved.as_deref(), Some("acme-project"));
    }

    #[test]
    fn resolve_project_name_returns_default_only_for_default_source() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());
        write_projects_yml(
            xdg.path(),
            r#"
default: fallback-project
mappings:
  acme: acme-project
"#,
        );

        let resolved = resolve_project_name("unknown", TriggerSource::Default).unwrap();

        assert_eq!(resolved.as_deref(), Some("fallback-project"));
    }

    #[test]
    fn resolve_project_name_returns_none_for_unmapped_non_default_source() {
        let xdg = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let _env = EnvGuard::isolated(xdg.path(), project.path());
        write_projects_yml(
            xdg.path(),
            r#"
default: fallback-project
mappings:
  acme: acme-project
"#,
        );

        let resolved = resolve_project_name("unknown", TriggerSource::GitRemote).unwrap();

        assert_eq!(resolved, None);
    }

    fn write_projects_yml(xdg: &std::path::Path, content: &str) {
        let dir = xdg.join("stripex");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("projects.yml"), content).unwrap();
    }
}
