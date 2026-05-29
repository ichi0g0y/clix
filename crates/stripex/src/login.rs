use std::process::{self, Command};

use clix_core::git;

use crate::config;
use crate::error::Error;

pub fn run(args: &[String]) -> Result<(), Error> {
    let help_mode = args.iter().any(|a| a == "--help" || a == "-h");
    let project = project_arg(args);

    let status = Command::new("stripe")
        .arg("login")
        .args(args)
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::StripeNotFound
            } else {
                Error::ExecFailed(e.to_string())
            }
        })?;

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
    if help_mode {
        return Ok(());
    }

    if let Some(project) = project {
        auto_bind(&project)?;
    }
    Ok(())
}

fn auto_bind(project: &str) -> Result<(), Error> {
    let owner = match git::get_remote_owner() {
        Ok(owner) => owner,
        Err(_) => return Ok(()),
    };

    let mut cfg = config::load()?;
    if cfg.mappings.contains_key(&owner) {
        return Ok(());
    }

    cfg.mappings.insert(owner.clone(), project.to_string());
    config::save(&cfg)?;
    eprintln!("stripex: bound git remote owner {owner} -> {project}");
    Ok(())
}

fn project_arg(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-p" || arg == "--project-name" {
            return args.get(i + 1).and_then(|value| non_empty(value));
        }
        if let Some(value) = arg.strip_prefix("-p=") {
            return non_empty(value);
        }
        if let Some(value) = arg.strip_prefix("--project-name=") {
            return non_empty(value);
        }
        i += 1;
    }
    None
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::project_arg;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_project_arg_forms() {
        assert_eq!(project_arg(&args(&["-p", "work"])).as_deref(), Some("work"));
        assert_eq!(
            project_arg(&args(&["--project-name", "work"])).as_deref(),
            Some("work")
        );
        assert_eq!(project_arg(&args(&["-p=work"])).as_deref(), Some("work"));
        assert_eq!(
            project_arg(&args(&["--project-name=work"])).as_deref(),
            Some("work")
        );
        assert_eq!(project_arg(&args(&["--help"])), None);
        assert_eq!(project_arg(&args(&["-p="])), None);
    }
}
