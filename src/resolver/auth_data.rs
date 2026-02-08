use crate::Headers;
use crate::resolver::{Error, Result};
use std::collections::HashMap;
/// `AuthData` specifies either how or the key itself for an authentication resolver call.
#[derive(Clone)]
pub enum AuthData {
	/// Specify the environment name to get the key value from.
	FromEnv(String),

	/// The key value itself.
	Key(String),

	/// OAuth Bearer token for providers that support `Authorization: Bearer` auth.
	BearerToken(String),

	/// Override headers and request url for unorthodox authentication schemes
	RequestOverride { url: String, headers: Headers },

	/// The key names/values when a credential has multiple pieces of credential information.
	/// This will be adapter-specific.
	/// NOTE: Not used yet.
	MultiKeys(HashMap<String, String>),
}

/// Constructors
impl AuthData {
	/// Create a new `AuthData` from an environment variable name.
	pub fn from_env(env_name: impl Into<String>) -> Self {
		AuthData::FromEnv(env_name.into())
	}

	/// Create a new `AuthData` from a single value.
	pub fn from_single(value: impl Into<String>) -> Self {
		AuthData::Key(value.into())
	}

	/// Create a new `AuthData` from multiple values.
	pub fn from_multi(data: HashMap<String, String>) -> Self {
		AuthData::MultiKeys(data)
	}
}

/// Getters
impl AuthData {
	/// Get the single value from the `AuthData`.
	pub fn single_key_value(&self) -> Result<String> {
		match self {
			// Overrides don't use an api key
			AuthData::RequestOverride { .. } => Ok(String::new()),
			// Bearer tokens return the token value
			AuthData::BearerToken(value) => Ok(value.to_string()),
			AuthData::FromEnv(env_name) => {
				// Get value from the environment name.
				let value = std::env::var(env_name).map_err(|_| Error::ApiKeyEnvNotFound {
					env_name: env_name.to_string(),
				})?;
				Ok(value)
			}
			AuthData::Key(value) => Ok(value.to_string()),
			_ => Err(Error::ResolverAuthDataNotSingleValue),
		}
	}
}

// region:    --- AuthData Std Impls

// Implement Debug to redact sensitive information.
impl std::fmt::Debug for AuthData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			// NOTE: Here we also redact for `FromEnv` in case the developer confuses this with a key.
			AuthData::FromEnv(_env_name) => write!(f, "AuthData::FromEnv(REDACTED)"),
			AuthData::Key(_) => write!(f, "AuthData::Single(REDACTED)"),
			AuthData::BearerToken(_) => write!(f, "AuthData::BearerToken(REDACTED)"),
			AuthData::MultiKeys(_) => write!(f, "AuthData::Multi(REDACTED)"),
			AuthData::RequestOverride { .. } => {
				write!(f, "AuthData::RequestOverride {{ url: REDACTED, headers: REDACTED }}")
			}
		}
	}
}

// endregion: --- AuthData Std Impls

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bearer_token_single_key_value() {
		let auth = AuthData::BearerToken("sk-ant-oat01-test-token".to_string());
		let value = auth.single_key_value().unwrap();
		assert_eq!(value, "sk-ant-oat01-test-token");
	}

	#[test]
	fn test_bearer_token_debug_redacted() {
		let auth = AuthData::BearerToken("secret-token".to_string());
		let debug = format!("{:?}", auth);
		assert_eq!(debug, "AuthData::BearerToken(REDACTED)");
		assert!(!debug.contains("secret-token"));
	}

	#[test]
	fn test_key_single_key_value() {
		let auth = AuthData::Key("test-api-key".to_string());
		let value = auth.single_key_value().unwrap();
		assert_eq!(value, "test-api-key");
	}

	#[test]
	fn test_request_override_returns_empty() {
		let auth = AuthData::RequestOverride {
			url: "https://example.com".to_string(),
			headers: Headers::default(),
		};
		let value = auth.single_key_value().unwrap();
		assert_eq!(value, "");
	}
}

// endregion: --- Tests
