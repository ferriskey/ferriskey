pub trait MaskStrategy {
    fn mask(value: &str) -> String;
}

/// Always ""******"."
pub struct FullMask;
impl MaskStrategy for FullMask {
    fn mask(_: &str) -> String {
        "******".into()
    }
}

/// Email masking: "******@domain.tld" when possible.
pub struct EmailMask;
impl MaskStrategy for EmailMask {
    fn mask(value: &str) -> String {
        value
            .split_once('@')
            .map(|(_, domain)| format!("******@{domain}"))
            .unwrap_or_else(|| "******.".into())
    }
}

/// Partial masking:
/// - keep first `prefix` chars
/// - keep last `suffix` chars
/// - middle replaced with '*'
///
/// Example: "123456789" with (2,2) => "12*****89"
pub struct PartialMask<const PREFIX: usize, const SUFFIX: usize>;
impl<const PREFIX: usize, const SUFFIX: usize> MaskStrategy for PartialMask<PREFIX, SUFFIX> {
    fn mask(value: &str) -> String {
        let chars: Vec<char> = value.chars().collect();
        let len = chars.len();

        if len == 0 {
            return "***".into();
        }

        // If too short, fall back to full mask
        if PREFIX + SUFFIX >= len {
            return "***".into();
        }

        let mut out = String::new();
        out.extend(chars.iter().take(PREFIX));
        out.extend(std::iter::repeat_n('*', len - PREFIX - SUFFIX));
        out.extend(chars.iter().skip(len - SUFFIX));
        out
    }
}

pub struct HashMask;
impl MaskStrategy for HashMask {
    fn mask(value: &str) -> String {
        let h = blake3::hash(value.as_bytes());
        let hex = h.to_hex().to_string();

        format!("hash:{}", &hex[..12])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_mask_is_constant() {
        assert_eq!(FullMask::mask("secret"), "******");
        assert_eq!(FullMask::mask(""), "******");
    }

    #[test]
    fn email_mask_handles_domain_and_missing_at() {
        assert_eq!(EmailMask::mask("user@example.com"), "******@example.com");
        assert_eq!(EmailMask::mask("not-an-email"), "******.");
    }

    #[test]
    fn partial_mask_handles_edges_and_middle() {
        assert_eq!(PartialMask::<2, 2>::mask(""), "***");
        assert_eq!(PartialMask::<2, 2>::mask("1234"), "***");
        assert_eq!(PartialMask::<2, 2>::mask("123456789"), "12*****89");
    }

    #[test]
    fn hash_mask_is_deterministic_and_short() {
        let masked = HashMask::mask("secret");
        let masked_again = HashMask::mask("secret");
        let other = HashMask::mask("different");

        assert!(masked.starts_with("hash:"));
        assert_eq!(masked.len(), "hash:".len() + 12);
        assert_eq!(masked, masked_again);
        assert_ne!(masked, other);
    }
}
