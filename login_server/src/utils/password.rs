use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PasswordError {
    HashingError(String),
    VerificationError(String),
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordError::HashingError(msg) => write!(f, "Password hashing error: {}", msg),
            PasswordError::VerificationError(msg) => {
                write!(f, "Password verification error: {}", msg)
            }
        }
    }
}

impl Error for PasswordError {}

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::HashingError(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| PasswordError::VerificationError(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password123";

        let hash = hash_password(password).expect("Failed to hash password");

        assert!(verify_password(password, &hash).expect("Failed to verify password"));

        assert!(!verify_password("wrong_password", &hash).expect("Failed to verify password"));
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = hash_password(password1).expect("Failed to hash password1");
        let hash2 = hash_password(password2).expect("Failed to hash password2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        let password = "same_password";

        let hash1 = hash_password(password).expect("Failed to hash password");
        let hash2 = hash_password(password).expect("Failed to hash password");

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        assert!(verify_password(password, &hash1).expect("Failed to verify hash1"));
        assert!(verify_password(password, &hash2).expect("Failed to verify hash2"));
    }
}
