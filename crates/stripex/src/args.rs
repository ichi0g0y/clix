#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    pub explicit_project: Option<String>,
    pub project_flag_present: bool,
    pub api_key_present: bool,
    pub dry_run: bool,
    pub raw: Vec<String>,
}

pub fn parse(args: &[String]) -> ParsedArgs {
    let mut out = ParsedArgs::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];

        if a == "--dry-run" {
            out.dry_run = true;
            i += 1;
            continue;
        }

        if let Some(value) = strip_flag(a, &["-p", "--project-name"]) {
            out.project_flag_present = true;
            out.raw.push(a.clone());
            if value.is_empty() && (a == "-p" || a == "--project-name") {
                if let Some(next) = args.get(i + 1) {
                    out.explicit_project = Some(next.clone());
                    out.raw.push(next.clone());
                    i += 2;
                    continue;
                }
            } else {
                out.explicit_project = Some(value.to_string());
            }
            i += 1;
            continue;
        }

        if let Some(value) = strip_flag(a, &["--api-key"]) {
            out.api_key_present = true;
            out.raw.push(a.clone());
            if value.is_empty()
                && a == "--api-key"
                && let Some(next) = args.get(i + 1)
            {
                out.raw.push(next.clone());
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        out.raw.push(a.clone());
        i += 1;
    }
    out
}

fn strip_flag<'a>(arg: &'a str, flags: &[&str]) -> Option<&'a str> {
    for &flag in flags {
        if arg == flag {
            return Some("");
        }
        if let Some(rest) = arg.strip_prefix(flag)
            && let Some(value) = rest.strip_prefix('=')
        {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn project_short_form_is_detected_and_forwarded() {
        let p = parse(&args(&["-p", "foo", "customers", "list"]));

        assert!(p.project_flag_present);
        assert_eq!(p.explicit_project.as_deref(), Some("foo"));
        assert_eq!(p.raw, args(&["-p", "foo", "customers", "list"]));
    }

    #[test]
    fn project_long_form_is_detected_and_forwarded() {
        let p = parse(&args(&["--project-name", "foo", "customers", "list"]));

        assert!(p.project_flag_present);
        assert_eq!(p.explicit_project.as_deref(), Some("foo"));
        assert_eq!(p.raw, args(&["--project-name", "foo", "customers", "list"]));
    }

    #[test]
    fn project_short_equals_form_is_detected_and_forwarded() {
        let p = parse(&args(&["-p=foo", "customers", "list"]));

        assert!(p.project_flag_present);
        assert_eq!(p.explicit_project.as_deref(), Some("foo"));
        assert_eq!(p.raw, args(&["-p=foo", "customers", "list"]));
    }

    #[test]
    fn project_long_equals_form_is_detected_and_forwarded() {
        let p = parse(&args(&["--project-name=foo", "customers", "list"]));

        assert!(p.project_flag_present);
        assert_eq!(p.explicit_project.as_deref(), Some("foo"));
        assert_eq!(p.raw, args(&["--project-name=foo", "customers", "list"]));
    }

    #[test]
    fn project_value_with_spaces_is_preserved() {
        let p = parse(&args(&["-p", "mijin co", "customers", "list"]));

        assert_eq!(p.explicit_project.as_deref(), Some("mijin co"));
        assert_eq!(p.raw, args(&["-p", "mijin co", "customers", "list"]));
    }

    #[test]
    fn api_key_is_detected_and_forwarded() {
        let p = parse(&args(&["--api-key", "sk_x", "customers", "list"]));

        assert!(p.api_key_present);
        assert_eq!(p.raw, args(&["--api-key", "sk_x", "customers", "list"]));
    }

    #[test]
    fn api_key_equals_form_is_detected_and_forwarded() {
        let p = parse(&args(&["--api-key=sk_x", "customers", "list"]));

        assert!(p.api_key_present);
        assert_eq!(p.raw, args(&["--api-key=sk_x", "customers", "list"]));
    }

    #[test]
    fn dry_run_is_detected_and_consumed() {
        let p = parse(&args(&["customers", "--dry-run", "list"]));

        assert!(p.dry_run);
        assert_eq!(p.raw, args(&["customers", "list"]));
    }

    #[test]
    fn other_args_keep_order() {
        let p = parse(&args(&[
            "customers",
            "list",
            "--limit",
            "3",
            "--api-key=sk_x",
            "-p=foo",
        ]));

        assert_eq!(
            p.raw,
            args(&[
                "customers",
                "list",
                "--limit",
                "3",
                "--api-key=sk_x",
                "-p=foo",
            ])
        );
    }

    #[test]
    fn bare_trailing_project_flag_is_forwarded_without_value() {
        let p = parse(&args(&["customers", "list", "-p"]));

        assert!(p.project_flag_present);
        assert!(p.explicit_project.is_none());
        assert_eq!(p.raw, args(&["customers", "list", "-p"]));
    }

    #[test]
    fn bare_project_flag_alone_suppresses_injection_and_is_forwarded() {
        let p = parse(&args(&["-p"]));

        assert!(p.project_flag_present);
        assert!(p.explicit_project.is_none());
        assert_eq!(p.raw, args(&["-p"]));
    }
}
