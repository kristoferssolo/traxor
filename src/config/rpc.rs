use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use thiserror::Error;
use transmission_rpc::types::BasicAuth;
use url::Url;

#[derive(Debug, Error)]
pub enum RpcConfigError {
    #[error("invalid rpc.url {url:?}: {source}")]
    InvalidUrl {
        url: String,
        #[source]
        source: url::ParseError,
    },

    #[error("invalid rpc.url {url:?}: scheme must be http or https")]
    UnsupportedScheme { url: String },

    #[error("rpc.username is set but rpc.password is missing")]
    MissingPassword,

    #[error("rpc.password is set but rpc.username is missing")]
    MissingUsername,
}

/// Transmission RPC connection settings.
///
/// `Debug` redacts the password so the config can be logged safely.
#[derive(Clone, Deserialize, Serialize)]
pub struct RpcConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            username: None,
            password: None,
        }
    }
}

impl RpcConfig {
    /// Validate the RPC URL and credentials.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid or only one of
    /// `username`/`password` is set.
    pub fn validate(&self) -> Result<(), RpcConfigError> {
        self.parsed_url()?;
        self.basic_auth()?;
        Ok(())
    }

    /// Parse the configured RPC URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL does not parse or is not http(s).
    pub fn parsed_url(&self) -> Result<Url, RpcConfigError> {
        let url = Url::parse(&self.url).map_err(|source| RpcConfigError::InvalidUrl {
            url: self.url.clone(),
            source,
        })?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(RpcConfigError::UnsupportedScheme {
                url: self.url.clone(),
            });
        }
        Ok(url)
    }

    /// Basic auth credentials, or `None` when neither username nor password is set.
    ///
    /// # Errors
    ///
    /// Returns an error when only one of `username`/`password` is set.
    pub fn basic_auth(&self) -> Result<Option<BasicAuth>, RpcConfigError> {
        match (&self.username, &self.password) {
            (Some(user), Some(password)) => Ok(Some(BasicAuth {
                user: user.clone(),
                password: password.clone(),
            })),
            (Some(_), None) => Err(RpcConfigError::MissingPassword),
            (None, Some(_)) => Err(RpcConfigError::MissingUsername),
            (None, None) => Ok(None),
        }
    }
}

impl Debug for RpcConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcConfig")
            .field("url", &self.url)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

fn default_url() -> String {
    "http://localhost:9091/transmission/rpc".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_matches, assert_ok, assert_some};

    fn rpc(username: Option<&str>, password: Option<&str>) -> RpcConfig {
        RpcConfig {
            username: username.map(Into::into),
            password: password.map(Into::into),
            ..RpcConfig::default()
        }
    }

    #[test]
    fn default_is_valid_without_auth() {
        let config = RpcConfig::default();
        assert_ok!(config.validate());
        assert_eq!(
            assert_ok!(config.parsed_url()).as_str(),
            "http://localhost:9091/transmission/rpc"
        );
    }

    #[test]
    fn builds_basic_auth_from_credentials() {
        let auth = assert_some!(assert_ok!(rpc(Some("user"), Some("secret")).basic_auth()));
        assert_eq!(auth.user, "user");
        assert_eq!(auth.password, "secret");
    }

    #[test]
    fn rejects_partial_credentials() {
        assert_matches!(
            rpc(Some("user"), None).validate(),
            Err(RpcConfigError::MissingPassword)
        );
        assert_matches!(
            rpc(None, Some("secret")).validate(),
            Err(RpcConfigError::MissingUsername)
        );
    }

    #[test]
    fn keeps_url_parse_error_as_source() {
        let config = RpcConfig {
            url: "not a url".into(),
            ..RpcConfig::default()
        };
        let err = assert_err!(config.validate());
        assert_matches!(err, RpcConfigError::InvalidUrl { .. });
        assert_some!(std::error::Error::source(&err));
    }

    #[test]
    fn rejects_non_http_scheme() {
        let config = RpcConfig {
            url: "localhost:9091/transmission/rpc".into(),
            ..RpcConfig::default()
        };
        assert_matches!(
            config.validate(),
            Err(RpcConfigError::UnsupportedScheme { .. })
        );
    }

    #[test]
    fn debug_redacts_password() {
        let output = format!("{:?}", rpc(Some("user"), Some("hunter2")));
        assert!(!output.contains("hunter2"));
        assert!(output.contains("<redacted>"));
        assert!(output.contains("user"));
    }
}
