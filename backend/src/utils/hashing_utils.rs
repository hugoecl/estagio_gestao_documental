use argon2_kdf::{Hash, Hasher};

#[inline(always)]
pub fn hash(password: &str) -> [u8; 48] {
    let hash = Hasher::new()
        .algorithm(argon2_kdf::Algorithm::Argon2id)
        .salt_length(16)
        .iterations(4)
        .memory_cost_kib(65536)
        .hash_length(32)
        .threads(4)
        .hash(password.as_bytes())
        .unwrap();
    let mut combined_bytes = [0u8; 48]; // 16 bytes for the salt and 32 bytes for the hash

    combined_bytes[..16].copy_from_slice(hash.salt_bytes());
    combined_bytes[16..].copy_from_slice(hash.as_bytes());

    combined_bytes
}

#[inline(always)]
pub fn verify(password: &str, combined_bytes: &[u8]) -> bool {
    let hash = Hash::from_parts(
        &combined_bytes[16..],
        &combined_bytes[..16],
        argon2_kdf::Algorithm::Argon2id,
        65536,
        4,
        4,
    );

    hash.verify(password.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_and_verify() {
        let password = "supersecretpassword";
        let hashed = hash(password);
        assert!(verify(password, &hashed));

        let wrong_password = "wrongpassword";
        assert!(!verify(wrong_password, &hashed));
    }

    #[test]
    fn test_verify_with_known_hash() {
        let password = "supersecretpassword";
        let combined_bytes: [u8; 48] = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0, // salt
            0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xda, 0xeb,
            0xfc, 0x0d, // hash
            0x1e, 0x2f, 0x3a, 0x4b, 0x5c, 0x6d, 0x7e, 0x8f, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef,
            0xfa, 0x0b, // hash
        ];
        assert!(!verify(password, &combined_bytes));
    }
}
