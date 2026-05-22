use base64::{engine::general_purpose::STANDARD, Engine};

#[derive(Debug, Clone)]
pub struct ChecksumInfo {
    pub algorithm: String,
    pub value: String,
}

impl ChecksumInfo {
    pub fn parse(header_value: &str) -> Option<Self> {
        let parts: Vec<&str> = header_value.split_whitespace().collect();

        if parts.len() != 2 {
            return None;
        }

        Some(Self {
            algorithm: parts[0].to_string(),
            value: parts[1].to_string(),
        })
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        let calculated = match self.algorithm.as_str() {
            "md5" => {
                let digest = md5::compute(data);
                STANDARD.encode(digest.as_ref())
            }
            "sha1" => {
                use sha1::{Digest, Sha1};
                let mut hasher = Sha1::new();
                hasher.update(data);
                let result = hasher.finalize();
                STANDARD.encode(result.as_slice())
            }
            "sha256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data);
                let result = hasher.finalize();
                STANDARD.encode(result.as_slice())
            }
            "sha512" => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(data);
                let result = hasher.finalize();
                STANDARD.encode(result.as_slice())
            }
            _ => return false,
        };

        calculated == self.value
    }
}
