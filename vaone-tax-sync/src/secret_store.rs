use std::fmt;

use keyring::Entry;

const SERVICE_NAME: &str = "com.congtyvacom.vaone-plugin.tax-sync";

#[derive(Debug)]
pub enum SecretStoreError {
    InvalidKey,
    EmptySecret,
    Keyring(keyring::Error),
}

impl fmt::Display for SecretStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "Secret key must not be empty"),
            Self::EmptySecret => write!(f, "Secret value must not be empty"),
            Self::Keyring(_) => write!(f, "Secure credential storage operation failed"),
        }
    }
}

impl std::error::Error for SecretStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Keyring(error) => Some(error),
            _ => None,
        }
    }
}

impl From<keyring::Error> for SecretStoreError {
    fn from(error: keyring::Error) -> Self {
        Self::Keyring(error)
    }
}

pub type Result<T> = std::result::Result<T, SecretStoreError>;

/// Stores secrets in the operating system's credential store.
/// SQLite should contain only the secret identifier, never the secret value.
#[derive(Debug, Clone, Copy, Default)]
pub struct SecretStore;

impl SecretStore {
    pub fn new() -> Self {
        Self
    }

    fn entry(&self, key: &str) -> Result<Entry> {
        let key = key.trim();

        if key.is_empty() {
            return Err(SecretStoreError::InvalidKey);
        }

        Entry::new(SERVICE_NAME, key).map_err(Into::into)
    }

    /// Creates or replaces a secret.
    pub fn set_secret(&self, key: &str, value: &str) -> Result<()> {
        if value.is_empty() {
            return Err(SecretStoreError::EmptySecret);
        }

        self.entry(key)?.set_password(value)?;
        Ok(())
    }

    /// Reads a secret. Returns an error if the key does not exist
    /// or the OS credential store cannot be accessed.
    pub fn get_secret(&self, key: &str) -> Result<String> {
        self.entry(key)?.get_password().map_err(Into::into)
    }

    /// Deletes a secret. Returns an error if deletion fails,
    /// including when the credential does not exist.
    pub fn delete_secret(&self, key: &str) -> Result<()> {
        self.entry(key)?.delete_credential()?;
        Ok(())
    }
}
