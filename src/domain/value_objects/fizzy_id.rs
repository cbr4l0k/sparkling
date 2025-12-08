use sqlx::decode;
use std::fmt;
use uuid::Uuid;

/// Fizzy uses UUIDv7 encoded as 25-character base36 strings.
/// This wrapper provides type safety and generation functionality.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FizzyId(String);

/// SQLX implementations so that FizzyId maps to the same SQL type as de default ones in the DB
impl sqlx::Type<sqlx::MySql> for FizzyId {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <[u8] as sqlx::Type<sqlx::MySql>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::MySql> for FizzyId {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<u8>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        // Convert Base36 String -> u128
        let num = Self::from_base36(&self.0)?;

        // Convert u128 -> Bytes (Big endian)
        let bytes = num.to_be_bytes();

        // Write bytes to SQLx buffer
        buf.extend_from_slice(&bytes);

        Ok(sqlx::encode::IsNull::No)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::MySql> for FizzyId {
    fn decode(
        value: <sqlx::MySql as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        // This expects bytes from the database (blob(16))
        // as seen by running `PRAGMA table_info(comments);`
        let bytes = <&[u8] as sqlx::Decode<sqlx::MySql>>::decode(value)?;
        if bytes.len() != 16 {
            return Err(format!(
                "Invalid FizzyId length in DB: Expected 16 bytes, got {}",
                bytes.len()
            )
            .into());
        }

        // Convert bytes -> u128
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        let num = u128::from_be_bytes(arr);

        // u128 -> Base36 String
        let base36 = Self::to_base36(num);
        Ok(Self(format!("{:0>25}", base36)))
    }
}

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

    /// Convert base 36 string back to u128 (requiered by sqlx::Encode)
    fn from_base36(s: &str) -> Result<u128, String> {
        let mut result: u128 = 0;

        for c in s.chars() {
            let val = match c {
                '0'..='9' => c as u128 - '0' as u128,
                'a'..='z' => c as u128 - 'a' as u128 + 10,
                'A'..='Z' => c as u128 - 'A' as u128 + 10,
                _ => return Err(format!("Invalid char: '{}' in Base36 string", c)),
            };
            result = result
                .checked_mul(36)
                .ok_or_else(|| "Base36 number to large for u128".to_string())?;

            result = result
                .checked_add(val)
                .ok_or_else(|| "Base36 number to large for u128".to_string())?;
        }
        Ok(result)
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
