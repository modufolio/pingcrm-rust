use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose, Engine as _};
use totp_lite::{totp_custom, Sha1};

const TOTP_DIGITS: u32 = 6;
const TOTP_STEP: u64 = 30;

pub struct TotpManager;

impl TotpManager {
    pub fn generate_secret() -> String {
        use uuid::Uuid;

        let uuid = Uuid::new_v4();
        let secret_bytes = uuid.as_bytes();

        general_purpose::STANDARD.encode(secret_bytes)
    }

    pub fn generate_code(secret: &str) -> AppResult<String> {
        let secret_bytes = Self::decode_secret(secret)?;
        let timestamp = Self::get_current_timestamp();

        let code = totp_custom::<Sha1>(TOTP_STEP, TOTP_DIGITS, &secret_bytes, timestamp);

        Ok(format!("{:0width$}", code, width = TOTP_DIGITS as usize))
    }

    pub fn verify_code(secret: &str, code: &str) -> AppResult<bool> {
        let secret_bytes = Self::decode_secret(secret)?;

        if code.len() != TOTP_DIGITS as usize {
            return Ok(false);
        }

        if !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::BadRequest("Invalid TOTP code format".to_string()));
        }

        let timestamp = Self::get_current_timestamp();

        if Self::check_code(&secret_bytes, code, timestamp) {
            return Ok(true);
        }

        if Self::check_code(&secret_bytes, code, timestamp - TOTP_STEP) {
            return Ok(true);
        }

        if Self::check_code(&secret_bytes, code, timestamp + TOTP_STEP) {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn generate_uri(secret: &str, account_name: &str, issuer: &str) -> AppResult<String> {
        let secret_bytes = Self::decode_secret(secret)?;
        let base32_secret = Self::encode_base32(&secret_bytes);

        let uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits={}&period={}",
            urlencoding::encode(issuer),
            urlencoding::encode(account_name),
            base32_secret,
            urlencoding::encode(issuer),
            TOTP_DIGITS,
            TOTP_STEP
        );

        Ok(uri)
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        use uuid::Uuid;

        (0..count)
            .map(|_| {
                let uuid = Uuid::new_v4();

                uuid.to_string().chars().take(8).collect()
            })
            .collect()
    }

    fn decode_secret(secret: &str) -> AppResult<Vec<u8>> {
        general_purpose::STANDARD
            .decode(secret)
            .map_err(|e| AppError::InternalError(format!("Failed to decode TOTP secret: {}", e)))
    }

    fn get_current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    fn check_code(secret_bytes: &[u8], provided_code: &str, timestamp: u64) -> bool {
        let expected_code = totp_custom::<Sha1>(TOTP_STEP, TOTP_DIGITS, secret_bytes, timestamp);
        expected_code == provided_code
    }

    fn encode_base32(bytes: &[u8]) -> String {
        const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut result = String::new();
        let mut bits = 0u32;
        let mut bit_count = 0;

        for &byte in bytes {
            bits = (bits << 8) | byte as u32;
            bit_count += 8;

            while bit_count >= 5 {
                bit_count -= 5;
                let index = ((bits >> bit_count) & 0x1F) as usize;
                result.push(BASE32_ALPHABET[index] as char);
            }
        }

        if bit_count > 0 {
            let index = ((bits << (5 - bit_count)) & 0x1F) as usize;
            result.push(BASE32_ALPHABET[index] as char);
        }

        while !result.len().is_multiple_of(8) {
            result.push('=');
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = TotpManager::generate_secret();
        assert!(!secret.is_empty());

        assert!(general_purpose::STANDARD.decode(&secret).is_ok());
    }

    #[test]
    fn test_generate_and_verify_code() {
        let secret = TotpManager::generate_secret();
        let code = TotpManager::generate_code(&secret).unwrap();

        assert_eq!(code.len(), TOTP_DIGITS as usize);
        assert!(TotpManager::verify_code(&secret, &code).unwrap());
    }

    #[test]
    fn test_invalid_code() {
        let secret = TotpManager::generate_secret();

        assert!(!TotpManager::verify_code(&secret, "000000").unwrap());

        assert!(!TotpManager::verify_code(&secret, "abc").unwrap());
    }

    #[test]
    fn test_generate_uri() {
        let secret = TotpManager::generate_secret();
        let uri = TotpManager::generate_uri(&secret, "user@example.com", "MyApp").unwrap();

        assert!(uri.starts_with("otpauth://totp/"));

        assert!(uri.contains("user%40example.com"));
        assert!(uri.contains("MyApp"));
        assert!(uri.contains("secret="));
    }

    #[test]
    fn test_generate_backup_codes() {
        let codes = TotpManager::generate_backup_codes(10);

        assert_eq!(codes.len(), 10);
        for code in codes {
            assert_eq!(code.len(), 8);
        }
    }

    #[test]
    fn test_base32_encoding() {
        let bytes = b"Hello";
        let encoded = TotpManager::encode_base32(bytes);

        assert!(encoded.starts_with("JBSWY3DP"));
    }
}
