use crate::pipeline::pass::Pass;
use serde::{de::DeserializeOwned, Serialize};


pub struct Pipeline<Config>
where
    Config: Serialize + DeserializeOwned,
{
    pub(crate) passes: Vec<Box<dyn Pass<Config>>>,
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
}


impl<Config> Default for Pipeline<Config>
where
    Config: Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}
