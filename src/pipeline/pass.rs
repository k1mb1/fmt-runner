use crate::pipeline::edit::Edit;
use serde::{de::DeserializeOwned, Serialize};
use tree_sitter::Node;

/// Base trait for all formatting passes.
///
/// A pass analyzes the AST and source code to produce a list of edits
/// that should be applied to format the code according to specific rules.
///
/// # Type Parameters
/// * `Config` - The configuration type for this pass
///
/// # Examples
/// ```ignore
/// struct MyPass;
///
/// impl Pass for MyPass {
///     type Config = MyConfig;
///
///     fn run(&self, config: &Self::Config, root: &Node, source: &str) -> Vec<Edit> {
///         // Analyze AST and return edits
///         vec![]
///     }
/// }
/// ```
pub trait Pass {
    /// The type of configuration for this pass
    type Config: Serialize + DeserializeOwned;

    /// Run the pass on the given AST and source code.
    ///
    /// # Arguments
    /// * `config` - The configuration for this pass
    /// * `root` - The root node of the AST
    /// * `source` - The source code
    ///
    /// # Returns
    /// A vector of edits to apply to the source code
    fn run(&self, config: &Self::Config, root: &Node, source: &str) -> Vec<Edit>;
}

/// Type-erased wrapper for passes to enable dynamic dispatch.
///
/// This trait allows storing passes with different associated types
/// in a single collection by erasing the associated type information.
pub trait ErasedPass<Config> {
    /// Run the pass with the given configuration.
    fn run(&self, config: &Config, root: &Node, source: &str) -> Vec<Edit>;
}

impl<T> ErasedPass<<T as Pass>::Config> for T
where
    T: Pass,
{
    fn run(&self, config: &<T as Pass>::Config, root: &Node, source: &str) -> Vec<Edit> {
        <T as Pass>::run(self, config, root, source)
    }
}

/// Structured trait for passes that work with concrete items.
///
/// This trait provides a higher-level abstraction for passes that follow
/// a common pattern: extract items from the AST, transform them according
/// to rules, and build the formatted output.
///
/// # Type Parameters
/// * `Config` - The configuration type
/// * `Item` - The type of items being formatted (e.g., imports, function arguments)
///
/// # Workflow
/// 1. `extract` - Find all targets in the AST
/// 2. `transform` - Modify items according to formatting rules
/// 3. `build` - Generate the formatted text from items
pub trait StructuredPass {
    /// The type of configuration
    type Config: Serialize + DeserializeOwned;
    /// The type of items being formatted
    type Item;

    /// Extract all edit targets from the AST.
    ///
    /// This method should traverse the AST and identify all locations
    /// that need formatting, along with the items they contain.
    ///
    /// # Arguments
    /// * `root` - The root node of the AST
    /// * `source` - The source code
    ///
    /// # Returns
    /// A vector of edits with items, each containing a range and items to be transformed
    fn extract(&self, root: &Node, source: &str) -> Vec<Edit<Self::Item>>;

    /// Transform the items according to formatting rules.
    ///
    /// This method can sort, filter, deduplicate, or otherwise modify
    /// the items before they are built into the final formatted text.
    ///
    /// # Arguments
    /// * `root` - The root node of the AST
    /// * `source` - The source code
    /// * `config` - The configuration
    /// * `items` - Mutable reference to the items to transform
    ///
    /// # Returns
    /// `Ok(())` on success, or an error message
    fn transform(
        &self,
        _root: &Node,
        _source: &str,
        _config: &Self::Config,
        _items: &mut Vec<Self::Item>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Build the formatted text from the items.
    ///
    /// This method should generate the final formatted string that
    /// will replace the original text in the edit range.
    ///
    /// # Arguments
    /// * `config` - The configuration
    /// * `items` - The items to format
    ///
    /// # Returns
    /// The formatted text
    fn build(&self, config: &Self::Config, items: &[Self::Item]) -> String;
}

impl<T> Pass for T
where
    T: StructuredPass,
{
    type Config = <T as StructuredPass>::Config;

    fn run(&self, config: &Self::Config, root: &Node, source: &str) -> Vec<Edit> {
        let mut edits = Vec::new();

        for mut target in self.extract(root, source) {
            if target.items.is_empty() {
                continue;
            }

            if let Err(err) = self.transform(root, source, config, &mut target.items) {
                eprintln!("Transform error in pass: {}", err);
                continue;
            }

            let content = self.build(config, &target.items);
            edits.push(target.with_content(content));
        }

        edits
    }
}
