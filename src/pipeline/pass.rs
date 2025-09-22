use crate::pipeline::edit::{Edit, EditTarget};
use serde::{de::DeserializeOwned, Serialize};
use tree_sitter::Node;


/// Базовый трейт для всех пассов форматирования
pub trait Pass<Config>
where
    Config: Serialize + DeserializeOwned,
{
    /// Получает AST и исходный код, возвращает список правок
    fn run(&self, config: &Config, root: &Node, source: &str) -> Vec<Edit>;
}


/// Структурированный трейт для пассов, которые работают с конкретными элементами
pub trait StructuredPass<Config>
where
    Config: Serialize + DeserializeOwned,
{
    type Item;

    /// Парсим AST и находим все целевые фрагменты для форматирования
    fn find_targets(&self, root: &Node, source: &str) -> Vec<EditTarget<Self::Item>>;

    /// Трансформируем элементы (сортировка, выравнивание, удаление и т.п.)
    fn transform(
        &self,
        _root: &Node,
        _source: &str,
        _config: &Config,
        _items: &mut Vec<Self::Item>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Собираем готовый код из элементов
    fn build(&self, items: &[Self::Item]) -> String;
}


impl<T, Config> Pass<Config> for T
where
    T: StructuredPass<Config>,
    Config: Serialize + DeserializeOwned,
{
    fn run(&self, config: &Config, root: &Node, source: &str) -> Vec<Edit> {
        let mut edits = Vec::new();

        for mut target in self.find_targets(root, source) {
            if target.items.is_empty() {
                continue;
            }

            if let Err(err) = self.transform(root, source, config, &mut target.items) {
                eprintln!("Rule error: {}", err);
                continue;
            }

            let content = self.build(&target.items);
            edits.push(Edit {
                range: target.range,
                content,
            });
        }

        edits
    }
}
