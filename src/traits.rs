use crate::error::ValidationError;

/// Core trait implemented by any single-value validator.
pub trait Validator<T: ?Sized> {
    /// Validates the given value, returning the first error encountered.
    fn validate(&self, value: &T) -> Result<(), ValidationError> {
        let mut errors = self.validate_all(value);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.remove(0))
        }
    }

    /// Validates the given value, returning all accumulated errors.
    fn validate_all(&self, value: &T) -> Vec<ValidationError>;
}
