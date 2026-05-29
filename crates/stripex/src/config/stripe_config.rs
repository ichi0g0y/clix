use std::fs;
use std::path::PathBuf;

use toml::Value;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripeProject {
    pub name: String,
    pub account_id: Option<String>,
    pub display_name: Option<String>,
}

pub fn stripe_config_path() -> Result<PathBuf, Error> {
    let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else {
        dirs::home_dir()
            .ok_or(Error::ConfigDirUnavailable)?
            .join(".config")
    };
    Ok(base.join("stripe").join("config.toml"))
}

pub fn load_projects() -> Result<Vec<StripeProject>, Error> {
    let path = stripe_config_path()?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(Error::ConfigParseError {
                path,
                msg: e.to_string(),
            });
        }
    };
    parse_projects(&content).map_err(|msg| Error::ConfigParseError { path, msg })
}

fn parse_projects(content: &str) -> Result<Vec<StripeProject>, String> {
    let value: Value = toml::from_str(content).map_err(|e| e.to_string())?;
    let Some(table) = value.as_table() else {
        return Ok(Vec::new());
    };

    let mut projects = Vec::new();
    for (name, section) in table {
        let Some(section) = section.as_table() else {
            continue;
        };
        projects.push(StripeProject {
            name: name.clone(),
            account_id: string_field(section, "account_id"),
            display_name: string_field(section, "display_name"),
        });
    }
    Ok(projects)
}

fn string_field(table: &toml::map::Map<String, Value>, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::test_support::EnvGuard;

    #[test]
    fn missing_stripe_config_returns_empty_projects() {
        let dir = TempDir::new().unwrap();
        let _env = EnvGuard::set_xdg(dir.path());

        assert!(load_projects().unwrap().is_empty());
    }

    #[test]
    fn loads_named_sections_and_skips_scalars_without_keys() {
        let dir = TempDir::new().unwrap();
        let _env = EnvGuard::set_xdg(dir.path());
        let stripe_dir = dir.path().join("stripe");
        fs::create_dir_all(&stripe_dir).unwrap();
        fs::write(
            stripe_dir.join("config.toml"),
            r#"
color = "off"

[default]
account_id = "acct_default"
display_name = "Default Account"
test_mode_api_key = "sk_test_do_not_surface"
live_mode_api_key = "sk_live_do_not_surface"

['mijin co']
account_id = "acct_mijin"
display_name = "Mijin Co"

[keychain]
account_id = "acct_keychain"
display_name = "Keychain Mode"
"#,
        )
        .unwrap();

        let projects = load_projects().unwrap();
        let debug = format!("{projects:?}");

        assert_eq!(projects.len(), 3);
        assert!(projects.iter().all(|p| !p.name.contains("sk_")));
        assert!(!debug.contains("sk_test_do_not_surface"));
        assert!(!debug.contains("sk_live_do_not_surface"));
        assert_eq!(projects[0].name, "default");
        assert_eq!(projects[0].account_id.as_deref(), Some("acct_default"));
        assert_eq!(projects[0].display_name.as_deref(), Some("Default Account"));
        let quoted = projects.iter().find(|p| p.name == "mijin co").unwrap();
        assert_eq!(quoted.account_id.as_deref(), Some("acct_mijin"));
        let keychain = projects.iter().find(|p| p.name == "keychain").unwrap();
        assert_eq!(keychain.display_name.as_deref(), Some("Keychain Mode"));
    }
}
