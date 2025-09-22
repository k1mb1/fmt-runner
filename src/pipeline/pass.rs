use crate::pipeline::edit::{Edit, EditTarget};
use serde::{de::DeserializeOwned, Serialize};
use tree_sitter::Node;


/// Базовый трейт для всех пассов форматирования
pub trait Pass<C>
where
    C: Serialize + DeserializeOwned,
{
    /// Получает AST и исходный код, возвращает список правок
    fn run(&self, config: &C, root: &Node, source: &String) -> Vec<Edit>;
}

/// Структурированный трейт для пассов, которые работают с конкретными элементами
pub trait StructuredPass<T>
where
    T: Serialize + DeserializeOwned,
{
    type Item;

    /// Парсим AST и находим все целевые фрагменты для форматирования
    fn find_targets(&self, root: &Node, source: &String) -> Vec<EditTarget<Self::Item>>;

    /// Трансформируем элементы (сортировка, выравнивание, удаление и т.п.)
    fn transform(
        &self,
        _root: &Node,
        _source: &String,
        _config: &T,
        _items: &mut Vec<Self::Item>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Собираем готовый код из элементов
    fn build(&self, items: &[Self::Item]) -> String;
}

impl<T, C> Pass<C> for T
where
    T: StructuredPass<C>,
    C: Serialize + DeserializeOwned,
{
    fn run(&self, config: &C, root: &Node, source: &String) -> Vec<Edit> {
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
