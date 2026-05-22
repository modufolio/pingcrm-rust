use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString};
use argon2::Argon2;

pub fn hash_password(plaintext: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|h| h.to_string())
}

pub fn verify_password(plaintext: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(plaintext.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn needs_rehash(hash: &str) -> bool {
    !hash.starts_with("$argon2id$")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_argon2() {
        let hash = hash_password("secret").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("secret", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn rejects_garbage() {
        assert!(!verify_password("secret", ""));
        assert!(!verify_password("secret", "not-a-hash"));
    }

    #[test]
    fn needs_rehash_flags_non_argon2() {
        let argon = hash_password("x").unwrap();
        assert!(!needs_rehash(&argon));
        assert!(needs_rehash("$2b$12$fakebcrypthashvalue"));
        assert!(needs_rehash("plaintext"));
    }
}
