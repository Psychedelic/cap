use cap_sdk_core::transaction::DetailValue;

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
