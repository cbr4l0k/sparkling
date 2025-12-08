use std::fmt;
use uuid::Uuid;

/// Fizzy uses UUIDv7 encoded as 25-character base36 strings.
/// This wrapper provides type safety and generation functionality.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FizzyId(String);

impl FizzyId {
    /// Create a FizzyId from an existing string (e.g., from database)
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a new UUIDv7-based FizzyId in base36 format
    pub fn generate() -> Self {
        let uuid = Uuid::now_v7();
        let bytes = uuid.as_bytes();
        let num = u128::from_be_bytes(*bytes);
        let base36 = Self::to_base36(num);
        Self(format!("{:0>25}", base36))
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Convert u128 to base36 lowercase string
    fn to_base36(mut num: u128) -> String {
        if num == 0 {
            return "0".to_string();
        }

        const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut result = Vec::new();

        while num > 0 {
            result.push(CHARS[(num % 36) as usize]);
            num /= 36;
        }

        result.reverse();
        String::from_utf8(result).unwrap()
    }
}

impl fmt::Display for FizzyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FizzyId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FizzyId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for FizzyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_fizzy_id_length() {
        let id = FizzyId::generate();
        assert_eq!(id.as_str().len(), 25);
    }

    #[test]
    fn test_fizzy_id_is_lowercase_alphanumeric() {
        let id = FizzyId::generate();
        assert!(id
            .as_str()
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_fizzy_id_uniqueness() {
        let id1 = FizzyId::generate();
        let id2 = FizzyId::generate();
        assert_ne!(id1, id2);
    }
}
