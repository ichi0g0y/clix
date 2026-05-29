use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Process-wide lock so tests that mutate environment variables don't
/// interleave. `Mutex<()>` because we only need ordering, not data.
pub(crate) static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Snapshot env vars stripex tests touch and restore them on Drop.
/// Uses [`ENV_LOCK`] for serialization. Recovers from poisoning so a
/// panic in one test does not cascade to the rest of the suite.
pub(crate) struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    old_xdg: Option<String>,
    restore_api_key: Option<Option<String>>,
    restore_secret_key: Option<Option<String>>,
    old_cwd: Option<std::path::PathBuf>,
}

impl EnvGuard {
    fn acquire() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Set only XDG_CONFIG_HOME. Used by config tests that don't touch
    /// auth env or cwd.
    pub fn set_xdg(xdg_config_home: &Path) -> Self {
        let lock = Self::acquire();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let old_api_key = std::env::var("STRIPE_API_KEY").ok();
        let old_secret_key = std::env::var("STRIPE_SECRET_KEY").ok();
        let old_cwd = std::env::current_dir().ok();

        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", xdg_config_home);
        }

        Self {
            _lock: lock,
            old_xdg,
            restore_api_key: Some(old_api_key),
            restore_secret_key: Some(old_secret_key),
            old_cwd,
        }
    }

    /// Full isolation: XDG_CONFIG_HOME, STRIPE_API_KEY,
    /// STRIPE_SECRET_KEY, and current_dir.
    #[allow(dead_code)]
    pub fn isolated(xdg_config_home: &Path, cwd: &Path) -> Self {
        let lock = Self::acquire();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let old_api_key = std::env::var("STRIPE_API_KEY").ok();
        let old_secret_key = std::env::var("STRIPE_SECRET_KEY").ok();
        let old_cwd = std::env::current_dir().ok();

        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", xdg_config_home);
            std::env::remove_var("STRIPE_API_KEY");
            std::env::remove_var("STRIPE_SECRET_KEY");
        }
        std::env::set_current_dir(cwd).ok();

        Self {
            _lock: lock,
            old_xdg,
            restore_api_key: Some(old_api_key),
            restore_secret_key: Some(old_secret_key),
            old_cwd,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.old_xdg {
                Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
            if let Some(prev) = self.restore_api_key.take() {
                match prev {
                    Some(value) => std::env::set_var("STRIPE_API_KEY", value),
                    None => std::env::remove_var("STRIPE_API_KEY"),
                }
            }
            if let Some(prev) = self.restore_secret_key.take() {
                match prev {
                    Some(value) => std::env::set_var("STRIPE_SECRET_KEY", value),
                    None => std::env::remove_var("STRIPE_SECRET_KEY"),
                }
            }
        }
        if let Some(cwd) = self.old_cwd.take() {
            let _ = std::env::set_current_dir(cwd);
        }
    }
}
