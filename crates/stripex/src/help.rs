pub const X_USAGE: &str = "usage: stripex x list\n\
                           \x20      stripex x bind <project> <trigger>\n\
                           \x20      stripex x unbind <trigger>\n\
                           \x20      stripex x use <project>\n\
                           \x20      stripex x whoami [<project>]\n";

pub const BARE_HINT: &str =
    "\nTip: run `stripex x` for stripex-specific subcommands (project / mapping management).\n";

pub const EXTRAS_SECTION: &str = "\nstripex extras (wrapper-specific subcommands):\n\
    \x20 stripex x list                       list Stripe CLI projects and trigger mappings\n\
    \x20 stripex x bind <project> <trigger>   map a trigger (git remote owner) to a project\n\
    \x20 stripex x unbind <trigger>           remove a trigger mapping\n\
    \x20 stripex x use <project>              set the default project\n\
    \x20 stripex x whoami [<project>]         show project authentication details\n\
    \x20 stripex x --help                     show this list\n";

pub fn is_top_level_help(args: &[String]) -> bool {
    matches!(args, [first] if matches!(first.as_str(), "--help" | "-h" | "help"))
}

pub fn is_x_help_arg(args: &[String]) -> bool {
    matches!(args, [first] if matches!(first.as_str(), "--help" | "-h"))
}

pub fn should_passthrough(args: &[String]) -> bool {
    match args {
        [first, ..] if matches!(first.as_str(), "logout" | "config" | "completion") => true,
        [first, _topic, ..] if first == "help" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{is_top_level_help, is_x_help_arg, should_passthrough};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn detects_top_level_help_only() {
        for values in [["--help"], ["-h"], ["help"]] {
            assert!(is_top_level_help(&args(&values)));
        }
        for values in [
            vec![],
            args(&["help", "customers"]),
            args(&["customers", "-h"]),
        ] {
            assert!(!is_top_level_help(&values), "{values:?}");
        }
    }

    #[test]
    fn detects_x_help_arg() {
        assert!(is_x_help_arg(&args(&["--help"])));
        assert!(is_x_help_arg(&args(&["-h"])));
        assert!(!is_x_help_arg(&args(&["help"])));
        assert!(!is_x_help_arg(&args(&["list", "--help"])));
    }

    #[test]
    fn passthrough_for_bootstrap_commands() {
        for values in [
            ["logout"].as_slice(),
            ["config"].as_slice(),
            ["completion"].as_slice(),
            ["help", "customers"].as_slice(),
        ] {
            assert!(should_passthrough(&args(values)), "{values:?}");
        }
    }

    #[test]
    fn login_and_x_are_not_passthrough() {
        assert!(!should_passthrough(&args(&["login"])));
        assert!(!should_passthrough(&args(&["x"])));
        assert!(!should_passthrough(&args(&["help"])));
    }
}
