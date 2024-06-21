use std::collections::HashMap;

/// This struct represents a storage of client id and their properties
pub struct CredentialManager {
    credentials: HashMap<String, String>,
}

impl CredentialManager {
    /// Returns an empty CredentialManager
    pub fn new() -> CredentialManager {
        CredentialManager {
            credentials: HashMap::new(),
        }
    }

    /// Add a pair username and password to credentials
    /// # Arguments
    ///
    /// * `username` - A string containing the username
    /// * `password` - A string containing the password
    ///
    pub fn add_credential(&mut self, username: &str, password: &str) {
        self.credentials
            .insert(username.to_string(), password.to_string());
    }

    /// Checks if a given username exists in the credential manager
    /// # Arguments
    ///
    /// * `username` - A string containing the username to search
    ///
    pub fn has_username(&self, username: &str) -> bool {
        self.credentials.contains_key(username)
    }

    /// Check username and password
    /// # Arguments
    ///
    /// * `username` - A string containing the username
    /// * `password` - A string containing the password
    ///
    pub fn is_valid(&self, username: &str, password: &str) -> bool {
        self.has_username(username) && password == self.credentials[username]
    }
}

#[cfg(test)]
mod tests {
    use crate::managers::credentialmanager;

    #[test]
    fn test_valid_username_and_password_succesful() {
        let mut credential_manager = credentialmanager::CredentialManager::new();
        let username = "user".to_string();
        let password = "pass".to_string();

        credential_manager.add_credential(&username, &password);
        assert_eq!(credential_manager.is_valid(&username, &password), true);
    }

    #[test]
    fn test_valid_username_and_password_fail() {
        let mut credential_manager = credentialmanager::CredentialManager::new();
        let username = "user".to_string();
        let password = "pass".to_string();
        let user_not_existing = "user_false".to_string();

        credential_manager.add_credential(&username, &password);
        assert_eq!(
            credential_manager.is_valid(&user_not_existing, &password),
            false
        );
    }
}
