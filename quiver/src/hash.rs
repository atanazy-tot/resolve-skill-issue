//! Content hashing for markers and drift detection.

use sha2::{Digest, Sha256};

/// The lowercase hex SHA-256 of the given text.
pub fn sha256_hex(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_stable_and_lowercase() {
        let hash = sha256_hex("quiver");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, sha256_hex("quiver"));
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }
}
