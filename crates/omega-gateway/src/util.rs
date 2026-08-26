//! Small shared helpers with no natural home in a single module.

/// Returns `n_bytes` of cryptographically random data as a lowercase hex string
/// (so the returned string is `2 * n_bytes` characters long).
pub(crate) fn random_hex(n_bytes: usize) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; n_bytes];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_hex_has_expected_length_and_charset() {
        let s = random_hex(8);
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn random_hex_is_actually_random() {
        assert_ne!(random_hex(8), random_hex(8));
    }
}
