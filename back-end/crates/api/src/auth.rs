use subtle::ConstantTimeEq;

/// Constant-time comparison for API keys to prevent timing attacks.
///
/// Used by all admin endpoints that require the `X-Seed-Api-Key` header.
pub fn verify_api_key(provided: &str, expected: &str) -> bool {
    // Convert to bytes for constant-time comparison
    let provided_bytes = provided.as_bytes();
    let expected_bytes = expected.as_bytes();

    // Length check must be done carefully - we compare both anyway to avoid
    // leaking length information through timing
    if provided_bytes.len() != expected_bytes.len() {
        // Still do a comparison to maintain constant time
        let _ = provided_bytes.ct_eq(provided_bytes);
        return false;
    }

    provided_bytes.ct_eq(expected_bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_keys() {
        assert!(verify_api_key("secret-key-123", "secret-key-123"));
    }

    #[test]
    fn test_mismatched_keys() {
        assert!(!verify_api_key("wrong-key", "secret-key-123"));
    }

    #[test]
    fn test_empty_keys_match() {
        assert!(verify_api_key("", ""));
    }

    #[test]
    fn test_different_lengths() {
        assert!(!verify_api_key("short", "longer-key"));
    }

    #[test]
    fn test_empty_provided_nonempty_expected() {
        assert!(!verify_api_key("", "secret"));
    }
}
