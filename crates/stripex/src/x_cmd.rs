use std::collections::BTreeMap;

use crate::args::ParsedArgs;
use crate::config::{self, stripe_config};
use crate::error::Error;
use crate::help;
use crate::resolve;
use crate::stripe_cli;

pub fn run(args: &[String]) -> Result<(), Error> {
    if args.is_empty() || help::is_x_help_arg(args) {
        clix_core::exec::write_or_exit_on_pipe_close(help::X_USAGE);
        return Ok(());
    }

    match args {
        [cmd] if cmd == "list" => list(),
        [cmd, project, trigger] if cmd == "bind" => bind(project, trigger),
        [cmd, trigger] if cmd == "unbind" => unbind(trigger),
        [cmd, project] if cmd == "use" => use_default(project),
        [cmd] if cmd == "whoami" => whoami(None),
        [cmd, project] if cmd == "whoami" => whoami(Some(project)),
        _ => {
            clix_core::exec::write_or_exit_on_pipe_close(help::X_USAGE);
            Ok(())
        }
    }
}

fn list() -> Result<(), Error> {
    let projects = stripe_config::load_projects()?;
    let cfg = config::load()?;

    if projects.is_empty() {
        println!("No Stripe CLI projects found.");
        println!("Run `stripe login` to create the first project.");
    } else {
        let by_project = mappings_by_project(&cfg.mappings);
        for project in &projects {
            let marker = if cfg.default.as_deref() == Some(project.name.as_str()) {
                "*"
            } else {
                " "
            };
            println!(
                "{marker} {}\taccount_id={}\tdisplay_name={}",
                project.name,
                project.account_id.as_deref().unwrap_or("-"),
                project.display_name.as_deref().unwrap_or("-")
            );
            if let Some(triggers) = by_project.get(&project.name) {
                println!("    mappings: {}", triggers.join(", "));
            }
        }
    }

    if !cfg.mappings.is_empty() {
        println!("\nmappings:");
        for (trigger, project) in &cfg.mappings {
            println!("    {trigger} -> {project}");
        }
    }
    if let Some(default) = &cfg.default {
        println!("\ndefault: {default}");
    }
    Ok(())
}

fn bind(project: &str, trigger: &str) -> Result<(), Error> {
    warn_if_project_missing(project)?;

    let mut cfg = config::load()?;
    cfg.mappings
        .insert(trigger.to_string(), project.to_string());
    config::save(&cfg)?;
    eprintln!("stripex: bound {trigger} -> {project}");
    Ok(())
}

fn unbind(trigger: &str) -> Result<(), Error> {
    let mut cfg = config::load()?;
    if cfg.mappings.remove(trigger).is_none() {
        return Err(Error::UnknownMapping(trigger.to_string()));
    }
    config::save(&cfg)?;
    eprintln!("stripex: unbound {trigger}");
    Ok(())
}

fn use_default(project: &str) -> Result<(), Error> {
    warn_if_project_missing(project)?;

    let mut cfg = config::load()?;
    cfg.default = Some(project.to_string());
    config::save(&cfg)?;
    eprintln!("stripex: default project set to \"{project}\"");
    Ok(())
}

fn whoami(project: Option<&str>) -> Result<(), Error> {
    let resolved = match project {
        Some(project) => Some(project.to_string()),
        None => {
            let parsed = ParsedArgs::default();
            let (trigger, source) = resolve::resolve_trigger(&parsed);
            resolve::resolve_project_name(&trigger, source)?
        }
    };

    let out = stripe_cli::whoami(resolved.as_deref())?;
    if let Some(project) = resolved.as_deref() {
        println!("project: {project}");
    } else {
        println!("project: (stripe default)");
    }
    println!(
        "account_id: {}",
        out.account_id.as_deref().unwrap_or("(unknown)")
    );
    println!(
        "display_name: {}",
        out.display_name.as_deref().unwrap_or("(unknown)")
    );
    println!("authenticated: {}", out.authenticated);
    println!(
        "keys: test={}, live={}",
        present_label(out.test_mode_key_present),
        present_label(out.live_mode_key_present)
    );
    Ok(())
}

fn mappings_by_project(mappings: &BTreeMap<String, String>) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (trigger, project) in mappings {
        out.entry(project.clone())
            .or_default()
            .push(trigger.clone());
    }
    out
}

fn warn_if_project_missing(project: &str) -> Result<(), Error> {
    let projects = stripe_config::load_projects()?;
    if !projects.iter().any(|p| p.name == project) {
        eprintln!(
            "stripex: warning: project \"{project}\" was not found in {}",
            stripe_config::stripe_config_path()?.display()
        );
    }
    Ok(())
}

fn present_label(present: bool) -> &'static str {
    if present { "present" } else { "absent" }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::test_support::EnvGuard;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn bind_records_mapping() {
        let dir = TempDir::new().unwrap();
        let _env = EnvGuard::set_xdg(dir.path());

        run(&args(&["bind", "work", "acme"])).unwrap();

        let cfg = config::load().unwrap();
        assert_eq!(cfg.mappings["acme"], "work");
    }

    #[test]
    fn unbind_removes_mapping() {
        let dir = TempDir::new().unwrap();
        let _env = EnvGuard::set_xdg(dir.path());
        run(&args(&["bind", "work", "acme"])).unwrap();

        run(&args(&["unbind", "acme"])).unwrap();

        let cfg = config::load().unwrap();
        assert!(!cfg.mappings.contains_key("acme"));
    }

    #[test]
    fn unbind_unknown_mapping_errors() {
        let dir = TempDir::new().unwrap();
        let _env = EnvGuard::set_xdg(dir.path());

        let err = run(&args(&["unbind", "missing"])).unwrap_err();

        assert!(matches!(err, Error::UnknownMapping(trigger) if trigger == "missing"));
    }

    #[test]
    fn use_sets_default_project() {
        let dir = TempDir::new().unwrap();
        let _env = EnvGuard::set_xdg(dir.path());

        run(&args(&["use", "work"])).unwrap();

        let cfg = config::load().unwrap();
        assert_eq!(cfg.default.as_deref(), Some("work"));
    }
}
