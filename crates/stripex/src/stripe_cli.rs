use std::io;
use std::process::Command;

use serde_json::Value;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Whoami {
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub authenticated: bool,
    pub test_mode_key_present: bool,
    pub live_mode_key_present: bool,
}

pub fn whoami(project: Option<&str>) -> Result<Whoami, Error> {
    let mut args = vec!["whoami", "--format", "json"];
    let mut project_args = Vec::new();
    if let Some(project) = project {
        project_args.push("-p");
        project_args.push(project);
    }
    args.extend(project_args);

    let output = Command::new("stripe").args(&args).output().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            Error::StripeNotFound
        } else {
            Error::ExecFailed(format!("failed to spawn stripe: {e}"))
        }
    })?;

    if !output.status.success() {
        let detail = command_detail(&output.stderr, &output.stdout);
        if is_unknown_format_error(&detail) {
            return Err(Error::ExecFailed(
                "this stripe CLI does not support `whoami --format json`; run `stripe whoami` directly or upgrade stripe".to_string(),
            ));
        }
        return Err(Error::ExecFailed(format!(
            "stripe {} failed: {detail}",
            args.join(" ")
        )));
    }

    parse_whoami(&String::from_utf8_lossy(&output.stdout))
}

pub fn parse_whoami(json: &str) -> Result<Whoami, Error> {
    let value: Value = serde_json::from_str(json).map_err(|e| {
        Error::ExecFailed(format!(
            "could not parse `stripe whoami --format json`: {e}"
        ))
    })?;

    let account_id = string_at(&value, &["account_id", "accountId"]);
    let display_name = string_at(&value, &["display_name", "displayName"]);
    let authenticated = bool_at(&value, "authenticated")
        .unwrap_or_else(|| account_id.is_some() || display_name.is_some());

    Ok(Whoami {
        account_id,
        display_name,
        authenticated,
        test_mode_key_present: value.get("test_mode_api_key").is_some()
            || value.get("testModeApiKey").is_some(),
        live_mode_key_present: value.get("live_mode_api_key").is_some()
            || value.get("liveModeApiKey").is_some(),
    })
}

fn command_detail(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }
    String::from_utf8_lossy(stdout).trim().to_string()
}

fn is_unknown_format_error(detail: &str) -> bool {
    let low = detail.to_lowercase();
    (low.contains("unknown") || low.contains("flag provided but not defined"))
        && (low.contains("--format") || low.contains("format"))
}

fn string_at(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .find_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn bool_at(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_authenticated_whoami_payload() {
        let parsed = parse_whoami(
            r#"{
                "account_id": "acct_123",
                "display_name": "Mijin Co",
                "authenticated": true,
                "test_mode_api_key": "redacted by stripe",
                "live_mode_api_key": "redacted by stripe"
            }"#,
        )
        .unwrap();

        assert_eq!(parsed.account_id.as_deref(), Some("acct_123"));
        assert_eq!(parsed.display_name.as_deref(), Some("Mijin Co"));
        assert!(parsed.authenticated);
        assert!(parsed.test_mode_key_present);
        assert!(parsed.live_mode_key_present);
    }

    #[test]
    fn parse_unauthenticated_whoami_payload() {
        let parsed = parse_whoami(r#"{"authenticated": false}"#).unwrap();

        assert_eq!(parsed.account_id, None);
        assert_eq!(parsed.display_name, None);
        assert!(!parsed.authenticated);
        assert!(!parsed.test_mode_key_present);
        assert!(!parsed.live_mode_key_present);
    }
}
