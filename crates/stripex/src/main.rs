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

    if parsed.raw.is_empty() {
        banner::print_stripex_banner()?;
        exec::write_or_exit_on_pipe_close(help::BARE_HINT);
        return run_stripe(cmd);
    }

    if help::is_top_level_help(&parsed.raw) {
        return run_stripe_with_extras(cmd);
    }

    if let [first, rest @ ..] = parsed.raw.as_slice()
        && first == "x"
    {
        return x_cmd::run(rest);
    }

    if let [first, rest @ ..] = parsed.raw.as_slice()
        && first == "login"
    {
        return login::run(rest);
    }

    if help::should_passthrough(&parsed.raw) {
        return run_stripe(cmd);
    }

    if parsed.dry_run {
        return resolve::print_dry_run(&parsed);
    }

    if parsed.project_flag_present {
        return run_stripe(cmd);
    }

    if resolve::has_stripe_env_key() || parsed.api_key_present {
        return run_stripe(cmd);
    }

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
    use super::is_version_command;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn detects_version_commands() {
        for values in [
            ["--version"].as_slice(),
            ["-v"].as_slice(),
            ["version"].as_slice(),
            ["version", "--help"].as_slice(),
        ] {
            assert!(is_version_command(&args(&values)));
        }
        assert!(!is_version_command(&args(&["--help"])));
        assert!(!is_version_command(&[]));
    }
}
