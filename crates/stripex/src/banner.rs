use std::env;

use clix_core::{banner, update};
use colored::Colorize;

use crate::{args, error::Error, resolve};

pub fn print_stripex_banner() -> Result<(), Error> {
    let ascii_art = [
        "███████╗████████╗██╗  ██╗",
        "██╔════╝╚══██╔══╝╚██╗██╔╝",
        "███████╗   ██║    ╚███╔╝",
        "╚════██║   ██║    ██╔██╗",
        "███████║   ██║   ██╔╝ ██╗",
        "╚══════╝   ╚═╝   ╚═╝  ╚═╝",
    ];

    let update_request = update::CheckRequest {
        repo_slug: "nantokaworks/clix",
        tool_name: "stripex",
        current_version: env!("CARGO_PKG_VERSION"),
        disable_env_var: "STRIPEX_NO_UPDATE_CHECK",
    };
    let update = update::check_for_update(&update_request).map(|info| banner::UpdateNotice {
        current: env!("CARGO_PKG_VERSION").to_string(),
        latest: info.latest,
        command: info.upgrade_cmd,
    });

    let banner = banner::Banner {
        ascii_art: &ascii_art,
        description: env!("CARGO_PKG_DESCRIPTION"),
        version: env!("CARGO_PKG_VERSION"),
        build_date: env!("STRIPEX_BUILD_DATE"),
        repository: env!("CARGO_PKG_REPOSITORY"),
        context_lines: context_lines(),
        update,
    };

    banner::print(&banner).map_err(|e| Error::ExecFailed(e.to_string()))
}

fn context_lines() -> Vec<String> {
    if let Some(name) = resolve::stripe_env_key() {
        return vec![format!(
            "{} {}",
            format!("{name}:").dimmed(),
            "(env override)".yellow()
        )];
    }

    let parsed = args::ParsedArgs::default();
    let (trigger, source) = resolve::resolve_trigger(&parsed);
    let trigger = if trigger.is_empty() {
        "(default)".to_string()
    } else {
        trigger
    };
    vec![
        format!("{} {}", "trigger:".dimmed(), trigger.yellow()),
        format!("{} {}", "source:".dimmed(), source_label(source).dimmed()),
    ]
}

fn source_label(source: resolve::TriggerSource) -> &'static str {
    match source {
        resolve::TriggerSource::ExplicitProject => "explicit project",
        resolve::TriggerSource::EnvApiKey => "env api key",
        resolve::TriggerSource::GitRemote => "git remote",
        resolve::TriggerSource::Default => "default",
    }
}
