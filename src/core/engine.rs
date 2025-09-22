use crate::parser::LanguageProvider;
use crate::parser::ParseState;
use crate::parser::Parser;
use crate::pipeline::Pipeline;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;


pub struct Engine<Language: LanguageProvider, Config>
where
    Config: Serialize + DeserializeOwned,
{
    pipeline: Pipeline<Config>,
    parser: Parser<Language>,
    _marker: PhantomData<(Language, Config)>,
}


impl<Language: LanguageProvider, Config> Engine<Language, Config>
where
    Config: Serialize + DeserializeOwned,
{
    pub fn new(pipeline: Pipeline<Config>) -> Self {
        Self {
            pipeline,
            parser: Parser::new(),
            _marker: PhantomData,
        }
    }

    /// Get a reference to the pipeline
    pub fn pipeline(&self) -> &Pipeline<Config> {
        &self.pipeline
    }

    /// Get a mutable reference to the pipeline
    pub fn pipeline_mut(&mut self) -> &mut Pipeline<Config> {
        &mut self.pipeline
    }

    /// Get a reference to the parser
    pub fn parser(&self) -> &Parser<Language> {
        &self.parser
    }

    /// Get a mutable reference to the parser
    pub fn parser_mut(&mut self) -> &mut Parser<Language> {
        &mut self.parser
    }

    fn run(&mut self, config: &Config, state: &mut ParseState) {
        if state.tree().is_none() {
            self.parser.parse(state);
        }

        for pass in self.pipeline.passes() {
            let root = state.tree().unwrap().root_node();
            let source = state.source();

            let mut edits = pass.run(config, &root, source);
            println!("Edits for pass: {:?}", edits);

            edits.sort_by(|a, b| b.range.0.cmp(&a.range.0));

            for edit in edits {
                self.parser
                    .apply_edit(state, edit.range.0, edit.range.1, &edit.content);
            }
        }
    }

    pub fn start(&mut self, config: &Config, codes: &[String]) {
        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.to_string());
            self.run(config, &mut state);

            println!("Final source for code {}:", i + 1);
            println!("{}", state.source());
            println!("---");
        }
    }
}
