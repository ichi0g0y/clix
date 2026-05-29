use std::fmt;
use std::path::PathBuf;

use clix_core::git::GitError;

#[derive(Debug)]
pub enum Error {
    ConfigDirUnavailable,
    ConfigParseError { path: PathBuf, msg: String },
    ConfigWriteError { path: PathBuf, msg: String },
    Config(String),
    Git(GitError),
    UnknownMapping(String),
    StripeNotFound,
    ExecFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigDirUnavailable => write!(f, "could not resolve the config directory"),
            Error::ConfigParseError { path, msg } => {
                write!(f, "failed to parse {}: {msg}", path.display())
            }
            Error::ConfigWriteError { path, msg } => {
                write!(f, "failed to write {}: {msg}", path.display())
            }
            Error::Config(msg) => write!(f, "config error: {msg}"),
            Error::Git(e) => write!(f, "{e}"),
            Error::UnknownMapping(trigger) => write!(f, "no project mapped to \"{trigger}\""),
            Error::StripeNotFound => write!(
                f,
                "stripe not found\n  Check: stripe --version\n  https://docs.stripe.com/stripe-cli"
            ),
            Error::ExecFailed(msg) => write!(f, "stripe execution failed: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Git(e) => Some(e),
            _ => None,
        }
    }
}

impl From<GitError> for Error {
    fn from(e: GitError) -> Self {
        Error::Git(e)
    }
}
