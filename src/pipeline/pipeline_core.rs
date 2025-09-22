use crate::pipeline::pass::Pass;
use serde::{de::DeserializeOwned, Serialize};


pub struct Pipeline<Config>
where
    Config: Serialize + DeserializeOwned,
{
    passes: Vec<Box<dyn Pass<Config>>>,
}


impl<Config> Pipeline<Config>
where
    Config: Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass<P>(&mut self, pass: P) -> &mut Self
    where
        P: Pass<Config> + 'static,
    {
        self.passes.push(Box::new(pass));
        self
    }

    /// Get a reference to the passes
    pub fn passes(&self) -> &[Box<dyn Pass<Config>>] {
        &self.passes
    }

    /// Get the number of passes
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Check if the pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }
}


impl<Config> Default for Pipeline<Config>
where
    Config: Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}
