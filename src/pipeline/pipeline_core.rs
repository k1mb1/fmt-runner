use crate::pipeline::Pass;
use crate::pipeline::pass::ErasedPass;

pub struct Pipeline<Config> {
    passes: Vec<Box<dyn ErasedPass<Config>>>,
}

impl<Config> Pipeline<Config> {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass<P>(&mut self, pass: P) -> &mut Self
    where
        P: Pass<Config = Config> + 'static,
    {
        self.passes.push(Box::new(pass));
        self
    }

    /// Get a reference to the passes
    pub fn passes(&self) -> &[Box<dyn ErasedPass<Config>>] {
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


impl<Config> Default for Pipeline<Config> {
    fn default() -> Self {
        Self::new()
    }
}
