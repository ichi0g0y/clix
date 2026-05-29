mod args;
mod banner;
mod config;
mod error;
mod help;
mod login;
mod resolve;
mod stripe_cli;
#[cfg(test)]
mod test_support;
mod x_cmd;

use std::env;
use std::process::{self, Command};

use clix_core::exec::{self, ExecError, exec_replace};

fn run() -> Result<(), error::Error> {
    let raw: Vec<String> = env::args().skip(1).collect();

    if is_version_command(&raw) {
        return banner::print_stripex_banner();
    }

    let parsed = args::parse(&raw);
    let mut cmd = Command::new("stripe");
    cmd.args(&parsed.raw);

    match route(&parsed) {
        Route::BareHint => {
            banner::print_stripex_banner()?;
            exec::write_or_exit_on_pipe_close(help::BARE_HINT);
            run_stripe(cmd)
        }
        Route::TopLevelHelp => run_stripe_with_extras(cmd),
        // `--dry-run` is inspected BEFORE the `x` / `login` wrapper
        // subcommands so a dry run can never trigger their side effects
        // (writing projects.yml, launching `stripe login`, etc.).
        Route::DryRun => resolve::print_dry_run(&parsed),
        Route::XCmd => x_cmd::run(&parsed.raw[1..]),
        Route::Login => login::run(&parsed.raw[1..]),
        Route::Passthrough => run_stripe(cmd),
        Route::Resolve => {
            let (trigger, source) = resolve::resolve_trigger(&parsed);
            match resolve::resolve_project_name(&trigger, source)? {
                Some(name) => {
                    let mut injected = Command::new("stripe");
                    injected.arg("-p").arg(&name);
                    injected.args(&parsed.raw);
                    run_stripe(injected)
                }
                None => run_stripe(cmd),
            }
        }
    }
}

/// Routing decision for a parsed invocation. Pure so the branch precedence
/// — notably `--dry-run` winning over the `x` / `login` wrapper subcommands —
/// is unit-testable without spawning `stripe`.
#[derive(Debug, PartialEq, Eq)]
enum Route {
    BareHint,
    TopLevelHelp,
    DryRun,
    XCmd,
    Login,
    Passthrough,
    Resolve,
}

fn route(parsed: &args::ParsedArgs) -> Route {
    if parsed.raw.is_empty() {
        return Route::BareHint;
    }
    if help::is_top_level_help(&parsed.raw) {
        return Route::TopLevelHelp;
    }
    // Inspect the stripex-only `--dry-run` flag before any side-effecting
    // branch (wrapper subcommands, passthrough exec).
    if parsed.dry_run {
        return Route::DryRun;
    }
    match parsed.raw.first().map(String::as_str) {
        Some("x") => return Route::XCmd,
        Some("login") => return Route::Login,
        _ => {}
    }
    if help::should_passthrough(&parsed.raw)
        || parsed.project_flag_present
        || resolve::has_stripe_env_key()
        || parsed.api_key_present
    {
        return Route::Passthrough;
    }
    Route::Resolve
}

fn run_stripe(cmd: Command) -> Result<(), error::Error> {
    exec_replace(cmd).map_err(map_exec_err)
}

fn run_stripe_with_extras(cmd: Command) -> Result<(), error::Error> {
    exec::run_with_trailer(cmd, help::EXTRAS_SECTION).map_err(map_exec_err)
}

fn map_exec_err(e: ExecError) -> error::Error {
    match e {
        ExecError::NotFound => error::Error::StripeNotFound,
        ExecError::Failed(msg) => error::Error::ExecFailed(msg),
    }
}

fn is_version_command(args: &[String]) -> bool {
    matches!(args, [first, ..] if matches!(first.as_str(), "--version" | "-v" | "version"))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("stripex: {e}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Route, is_version_command, route};
    use crate::args;

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn route_of(values: &[&str]) -> Route {
        route(&args::parse(&argv(values)))
    }

    #[test]
    fn detects_version_commands() {
        for values in [
            ["--version"].as_slice(),
            ["-v"].as_slice(),
            ["version"].as_slice(),
            ["version", "--help"].as_slice(),
        ] {
            assert!(is_version_command(&argv(values)));
        }
        assert!(!is_version_command(&argv(&["--help"])));
        assert!(!is_version_command(&[]));
    }

    #[test]
    fn dry_run_precedes_wrapper_subcommands() {
        // Regression: `--dry-run` must be inspected before `x` / `login` so a
        // dry run cannot write projects.yml or launch `stripe login`.
        assert_eq!(
            route_of(&["--dry-run", "x", "bind", "work", "acme"]),
            Route::DryRun
        );
        assert_eq!(
            route_of(&["--dry-run", "login", "-p", "work"]),
            Route::DryRun
        );
        assert_eq!(route_of(&["--dry-run", "logout"]), Route::DryRun);
        assert_eq!(route_of(&["--dry-run", "customers", "list"]), Route::DryRun);
    }

    #[test]
    fn routes_wrapper_help_and_bare() {
        assert_eq!(route_of(&["x", "list"]), Route::XCmd);
        assert_eq!(route_of(&["login"]), Route::Login);
        assert_eq!(route_of(&["logout"]), Route::Passthrough);
        assert_eq!(route_of(&[]), Route::BareHint);
        assert_eq!(route_of(&["--help"]), Route::TopLevelHelp);
    }

    #[test]
    fn resolve_route_without_overrides() {
        let tmp = tempfile::tempdir().unwrap();
        // Clears STRIPE_API_KEY / STRIPE_SECRET_KEY so `has_stripe_env_key`
        // is deterministic regardless of the developer's environment.
        let _guard = crate::test_support::EnvGuard::isolated(tmp.path(), tmp.path());
        assert_eq!(route_of(&["customers", "list"]), Route::Resolve);
        assert_eq!(
            route_of(&["-p", "foo", "customers", "list"]),
            Route::Passthrough
        );
    }
}
