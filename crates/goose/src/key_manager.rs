use keyring::Entry;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyManagerError {
    #[error("Failed to access keyring: {0}")]
    KeyringAccess(String),

    #[error("Failed to save to keyring: {0}")]
    KeyringSave(String),

    #[error("Failed to access environment variable: {0}")]
    EnvVarAccess(String),
}

impl From<keyring::Error> for KeyManagerError {
    fn from(err: keyring::Error) -> Self {
        KeyManagerError::KeyringAccess(err.to_string())
    }
}

impl From<env::VarError> for KeyManagerError {
    fn from(err: env::VarError) -> Self {
        KeyManagerError::EnvVarAccess(err.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeyRetrievalStrategy {
    /// Only look in environment variables
    EnvironmentOnly,
    /// Only look in system keyring
    KeyringOnly,
    /// Try keyring first, then environment variables (default behavior)
    Both,
}

impl Default for KeyRetrievalStrategy {
    fn default() -> Self {
        Self::Both
    }
}

pub fn get_keyring_secret(
    key_name: &str,
    strategy: KeyRetrievalStrategy,
) -> Result<String, Box<dyn std::error::Error>> {
    let kr = Entry::new("goose", key_name)?;
    match strategy {
        KeyRetrievalStrategy::EnvironmentOnly => env::var(key_name)
            .map_err(|e| Box::new(KeyManagerError::from(e)) as Box<dyn std::error::Error>),
        KeyRetrievalStrategy::KeyringOnly => kr
            .get_password()
            .map_err(|e| Box::new(KeyManagerError::from(e)) as Box<dyn std::error::Error>),
        KeyRetrievalStrategy::Both => {
            // Try environment first, then keyring
            env::var(key_name).or_else(|_| {
                kr.get_password().map_err(|_| {
                    Box::new(KeyManagerError::EnvVarAccess(format!(
                        "Could not find {} key in keyring or environment variables",
                        key_name
                    ))) as Box<dyn std::error::Error>
                })
            })
        }
    }
}

pub fn save_to_keyring(key_name: &str, key_val: &str) -> std::result::Result<(), KeyManagerError> {
    let kr = Entry::new("goose", key_name)?;
    kr.set_password(key_val).map_err(KeyManagerError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ENV_PREFIX: &str = "GOOSE_TEST_";

    fn cleanup_env(key: &str) {
        std::env::remove_var(key);
    }

    fn cleanup_keyring(key: &str) -> Result<(), KeyManagerError> {
        let kr = Entry::new("goose", key)?;
        kr.delete_credential().map_err(KeyManagerError::from)
    }

    #[test]
    fn test_get_key_environment_only() {
        let key_name = format!("{}{}", TEST_ENV_PREFIX, "ENV_KEY");
        std::env::set_var(&key_name, "test_value");

        let result = get_keyring_secret(&key_name, KeyRetrievalStrategy::EnvironmentOnly);
        assert_eq!(result.unwrap(), "test_value");

        cleanup_env(&key_name);
    }

    #[test]
    fn test_get_key_environment_only_missing() {
        let key_name = format!("{}{}", TEST_ENV_PREFIX, "MISSING_KEY");

        let result = get_keyring_secret(&key_name, KeyRetrievalStrategy::EnvironmentOnly);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_key_keyring_only() {
        let key_name = format!("{}{}", TEST_ENV_PREFIX, "KEYRING_KEY");

        // First save a value
        save_to_keyring(&key_name, "test_value").unwrap();

        let result = get_keyring_secret(&key_name, KeyRetrievalStrategy::KeyringOnly);
        assert_eq!(result.unwrap(), "test_value");

        cleanup_keyring(&key_name).unwrap();
    }

    #[test]
    fn test_get_key_both() {
        let key_name = format!("{}{}", TEST_ENV_PREFIX, "BOTH_KEY");

        // Test environment first
        std::env::set_var(&key_name, "env_value");
        let result = get_keyring_secret(&key_name, KeyRetrievalStrategy::Both);
        assert_eq!(result.unwrap(), "env_value");

        // Test keyring takes precedence
        save_to_keyring(&key_name, "keyring_value").unwrap();
        let result = get_keyring_secret(&key_name, KeyRetrievalStrategy::Both);
        assert_eq!(result.unwrap(), "env_value"); // Environment still takes precedence

        cleanup_env(&key_name);
        cleanup_keyring(&key_name).unwrap();
    }

    #[test]
    fn test_save_to_keyring() {
        let key_name = format!("{}{}", TEST_ENV_PREFIX, "SAVE_KEY");

        let result = save_to_keyring(&key_name, "test_value");
        assert!(result.is_ok());

        // Verify the value was saved
        let kr = Entry::new("goose", &key_name).unwrap();
        assert_eq!(kr.get_password().unwrap(), "test_value");

        cleanup_keyring(&key_name).unwrap();
    }
}