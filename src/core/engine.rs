use crate::parser::LanguageProvider;
use crate::parser::ParseState;
use crate::parser::Parser;
use crate::pipeline::Pipeline;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;


pub struct Engine<'a, L: LanguageProvider, C>
where
    C: Serialize + DeserializeOwned + 'a,
{
    pub pipeline: Pipeline<'a, C>,
    pub parser: Parser<L>,
    _marker: PhantomData<(L, C)>,
}

impl<'a, L: LanguageProvider, C> Engine<'a, L, C>
where
    C: Serialize + DeserializeOwned + 'a,
{
    pub fn new(pipeline: Pipeline<'a, C>) -> Self {
        Self {
            pipeline,
            parser: Parser::new(),
            _marker: PhantomData,
        }
    }

    fn run(&mut self, config: &C, state: &mut ParseState) {
        if state.tree().is_none() {
            self.parser.parse(state);
        }

        for pass in &self.pipeline.passes {
            let root = state.tree().unwrap().root_node();
            let source = state.source();

            let mut edits = pass.run(config, &root, &source);
            println!("Edits for pass: {:?}", edits);

            edits.sort_by(|a, b| b.range.0.cmp(&a.range.0));

            for edit in edits {
                self.parser
                    .apply_edit(state, edit.range.0, edit.range.1, &edit.content);
            }
        }
    }

    pub fn start(&mut self, config: &C, codes: &[String]) {
        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.to_string());
            self.run(config, &mut state);

            println!("Final source for code {}:", i + 1);
            println!("{}", state.source());
            println!("---");
        }
    }
}
