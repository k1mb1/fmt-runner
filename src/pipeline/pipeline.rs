use crate::pipeline::pass::Pass;
use serde::{de::DeserializeOwned, Serialize};


pub struct Pipeline<'a, C>
where
    C: Serialize + DeserializeOwned + 'a,
{
    pub passes: Vec<Box<dyn Pass<C> + 'a>>,
}

impl<'a, C> Pipeline<'a, C>
where
    C: Serialize + DeserializeOwned + 'a,
{
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass<P>(&mut self, pass: P) -> &mut Self
    where
        P: Pass<C> + 'a,
    {
        self.passes.push(Box::new(pass));
        self
    }
}
