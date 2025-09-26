use crate::pipeline::edit::{Edit, EditTarget};
use serde::{de::DeserializeOwned, Serialize};
use tree_sitter::Node;

/// Базовый трейт для всех пассов форматирования
pub trait Pass {
    /// Тип конфигурации для пасса
    type Config: Serialize + DeserializeOwned;

    /// Получает AST и исходный код, возвращает список правок
    fn run(&self, config: &Self::Config, root: &Node, source: &str) -> Vec<Edit>;
}

/// Обёртка для динамической диспетчеризации без привязки к ассоциированному типу в объекте
pub trait ErasedPass<Config> {
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

/// Структурированный трейт для пассов, которые работают с конкретными элементами
pub trait StructuredPass {
    /// Тип конфигурации
    type Config: Serialize + DeserializeOwned;
    /// Тип элемента (узла) форматирования
    type Item;

    /// Парсим AST и находим все целевые фрагменты для форматирования
    fn find_targets(&self, root: &Node, source: &str) -> Vec<EditTarget<Self::Item>>;

    /// Трансформируем элементы (сортировка, выравнивание, удаление и т.п.)
    fn transform(
        &self,
        _root: &Node,
        _source: &str,
        _config: &Self::Config,
        _items: &mut Vec<Self::Item>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Собираем готовый код из элементов
    fn build(&self, _config: &Self::Config, items: &[Self::Item]) -> String;
}

impl<T> Pass for T
where
    T: StructuredPass,
{
    type Config = <T as StructuredPass>::Config;

    fn run(&self, config: &Self::Config, root: &Node, source: &str) -> Vec<Edit> {
        let mut edits = Vec::new();

        for mut target in self.find_targets(root, source) {
            if target.items.is_empty() {
                continue;
            }

            if let Err(err) = self.transform(root, source, config, &mut target.items) {
                eprintln!("Rule error: {}", err);
                continue;
            }

            let content = self.build(config, &target.items);
            edits.push(Edit {
                range: target.range,
                content,
            });
        }

        edits
    }
}
