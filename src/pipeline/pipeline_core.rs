use crate::pipeline::pass::ErasedPass;
use crate::pipeline::Pass;

/// A pipeline of formatting passes that are applied sequentially.
///
/// The pipeline maintains an ordered collection of passes that will be
/// executed in sequence to transform source code. Each pass receives
/// the configuration and produces a set of edits.
///
/// # Type Parameters
/// * `Config` - The configuration type shared by all passes in the pipeline
///
/// # Examples
/// ```ignore
/// let mut pipeline = Pipeline::new();
/// pipeline.add_pass(MyFirstPass);
/// pipeline.add_pass(MySecondPass);
/// ```
pub struct Pipeline<Config> {
    passes: Vec<Box<dyn ErasedPass<Config>>>,
}

impl<Config> Pipeline<Config> {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Add a pass to the pipeline.
    ///
    /// Passes are executed in the order they are added.
    ///
    /// # Arguments
    /// * `pass` - The pass to add to the pipeline
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    ///
    /// # Examples
    /// ```ignore
    /// let mut pipeline = Pipeline::new();
    /// pipeline
    ///     .add_pass(FirstPass)
    ///     .add_pass(SecondPass);
    /// ```
    pub fn add_pass<P>(&mut self, pass: P) -> &mut Self
    where
        P: Pass<Config = Config> + 'static,
    {
        self.passes.push(Box::new(pass));
        self
    }

    /// Get a reference to the passes in this pipeline.
    ///
    /// # Returns
    /// A slice of boxed erased passes
    pub fn passes(&self) -> &[Box<dyn ErasedPass<Config>>] {
        &self.passes
    }

    /// Get the number of passes in the pipeline.
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Check if the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }
}

impl<Config> Default for Pipeline<Config> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyConfig;

    impl serde::Serialize for DummyConfig {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_unit()
        }
    }

    impl<'de> serde::Deserialize<'de> for DummyConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_unit(DummyConfigVisitor)
        }
    }

    struct DummyConfigVisitor;

    impl<'de> serde::de::Visitor<'de> for DummyConfigVisitor {
        type Value = DummyConfig;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("unit")
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(DummyConfig)
        }
    }

    #[test]
    fn test_new_pipeline_is_empty() {
        let pipeline: Pipeline<DummyConfig> = Pipeline::new();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);
    }

    #[test]
    fn test_default_pipeline_is_empty() {
        let pipeline: Pipeline<DummyConfig> = Pipeline::default();
        assert!(pipeline.is_empty());
    }
}
