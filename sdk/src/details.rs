use cap_sdk_core::transaction::DetailValue;

/// Allows converting an entire type into details for
/// a Cap event.
///
/// Use the `cap-standards` crate for premade standard-compliant
/// details structs with this trait already implemented for them.
pub trait IntoDetails {
    fn into_details(self) -> Vec<(String, DetailValue)>;
}

/// Allows trying to convert an event's details into
/// a struct.
///
/// Use the `cap-standards` crate for premade standard-compliant
/// details structs with this trait already implemented for them.
pub trait TryFromDetails: Sized {
    fn try_from_details(details: &Vec<(String, DetailValue)>) -> Result<Self, ()>;
}

/// Allows creating details for an event in an ergonomic fashion.
#[derive(Debug, Default)]
pub struct DetailsBuilder {
    inner: Vec<(String, DetailValue)>,
}

impl DetailsBuilder {
    /// Creates a new, empty builder.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new element.
    ///
    /// # Panics
    /// Panics if the key is already set.
    #[inline(always)]
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<DetailValue>) -> &mut Self {
        self.inner.push((key.into(), value.into()));

        self
    }

    #[inline(always)]
    pub fn build(self) -> Vec<(String, DetailValue)> {
        self.inner
    }
}
