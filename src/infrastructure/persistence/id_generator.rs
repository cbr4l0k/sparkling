use crate::domain::value_objects::FizzyId;

/// Generator for Fizzy-compatible IDs (UUIDv7 in base36 format)
pub struct FizzyIdGenerator;

impl FizzyIdGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a new Fizzy-compatible ID
    pub fn generate(&self) -> FizzyId {
        FizzyId::generate()
    }
}

impl Default for FizzyIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
