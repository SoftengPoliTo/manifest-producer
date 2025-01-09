use sha2::{Digest, Sha256};

/// Calculates the SHA-256 digest of a given byte buffer.
///
/// # Overview
/// This function takes a byte buffer (e.g., the contents of an ELF file) and computes its SHA-256 digest.
/// The resulting digest is returned as a hexadecimal string.
///
/// # Arguments
/// - `buffer`: A byte slice representing the data for which the digest is to be calculated.
///
/// # Returns
/// - A `String` containing the hexadecimal representation of the SHA-256 digest.
///
/// # Errors
/// - This function does not produce any errors directly.
///
#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn calculate_digest(buffer: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(buffer);
    let digest = hasher.finalize();
    format!("{digest:x}")
}

/// Compares two SHA-256 digest strings and returns a boolean indicating whether they match.
///
/// # Overview
/// This function compares two hexadecimal digest strings (e.g., SHA-256 hashes) and returns `true`
/// if they are equal, otherwise returns `false`. It is useful for verifying if two files have the same content.
///
/// # Arguments
/// - `digest1`: The first SHA-256 digest as a string.
/// - `digest2`: The second SHA-256 digest as a string.
///
/// # Returns
/// - A `bool` indicating whether the two digests match. Returns `true` if they are identical, otherwise `false`.
///
/// # Errors
/// - This function does not produce any errors since it's a direct string comparison.
///
#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn compare_digests(digest1: &str, digest2: &str) -> bool {
    digest1 == digest2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_digest() {
        let input = b"Hello, world!";
        let expected_digest = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"; // SHA-256 of "Hello, world!"
        let result = calculate_digest(input);

        assert_eq!(result, expected_digest);
    }

    #[test]
    fn test_compare_digests() {
        let digest1 = "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda93f3f7bc1c65b8c93a5";
        let digest2 = "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda93f3f7bc1c65b8c93a5";
        let digest3 = "c4ca4238a0b923820dcc509a6f75849b";

        assert!(compare_digests(digest1, digest2));
        assert!(!compare_digests(digest1, digest3));
    }
}
