k1mb1: Проанализируй fmt-runner и скажы Как улучшить эту библиотеку

GitHub Copilot: Я проанализирую библиотеку fmt-runner, чтобы дать вам рекомендации по улучшению. Начну с изучения основных файлов проекта. 

Отлично! Я проанализировал вашу библиотеку **fmt-runner**. Это интересный проект для создания форматтеров кода на базе Tree-sitter. Вот подробные рекомендации по улучшению:

## 🎯 **Основные рекомендации по улучшению**

### 1. **Документация и примеры** 📚

**Критическая проблема:** Отсутствует README.md

**Рекомендации:**
- Создайте подробный README.md с описанием проекта, примерами использования и архитектурой
- Добавьте документацию по созданию собственных passes
- Создайте больше примеров для разных языков
- Добавьте tutorial по интеграции с реальными tree-sitter грамматиками

### 2. **Тестирование** 🧪

**Проблемы:**
- Минимальное покрытие тестами (только базовые тесты в pipeline_core.rs и mod.rs)
- Отсутствуют интеграционные тесты
- Нет тестов для `Engine`, `Parser`, `Pass` реализаций

**Рекомендации:**
- Добавьте unit-тесты для всех модулей
- Создайте integration тесты с реальными tree-sitter грамматиками
- Добавьте property-based тесты (используя `proptest` или `quickcheck`)
- Добавьте benchmark тесты для производительности

### 3. **Обработка ошибок** ⚠️

**Проблемы:**
- В parser_core.rs используется `.expect()` при загрузке грамматики
- В `Engine::apply_edit` некорректная обработка позиций (используются байты вместо строк/столбцов)
- Недостаточная валидация входных данных

**Рекомендации:**
```rust
// Вместо:
ts_parser.set_language(&Language::language())
    .expect("Error loading grammar");

// Использовать:
ts_parser.set_language(&Language::language())
    .map_err(|e| ParserError::LanguageLoadError(e))?;
```

### 4. **Архитектурные улучшения** 🏗️

**Проблемы:**
- `InputEdit` в parser_core.rs использует неправильные координаты (только байты, а нужны строки и столбцы)
- Отсутствует параллельная обработка файлов
- Нет механизма кэширования парсинга

**Рекомендации:**

#### a) Исправьте вычисление позиций для tree-sitter:
```rust
// Добавьте helper функцию для конвертации байтов в Point
fn byte_to_point(source: &str, byte: usize) -> tree_sitter::Point {
    let before = &source[..byte.min(source.len())];
    let row = before.matches('\n').count();
    let column = before.split('\n').last().unwrap_or("").len();
    tree_sitter::Point { row, column }
}
```

#### b) Добавьте параллельную обработку файлов:
```rust
use rayon::prelude::*;

pub fn format_parallel(&mut self, config: &C, codes: &[String], files: &[PathBuf]) 
    -> Result<Vec<PathBuf>, std::io::Error> {
    codes.par_iter()
        .zip(files.par_iter())
        .map(|(code, file)| {
            // процесс форматирования
        })
        .collect()
}
```

### 5. **API улучшения** 🔧

**Рекомендации:**

#### a) Добавьте builder для `Engine`:
```rust
impl<Language: LanguageProvider, C> Engine<Language, C> {
    pub fn builder() -> EngineBuilder<Language, C> {
        EngineBuilder::new()
    }
}
```

#### b) Добавьте streaming API для больших файлов:
```rust
pub fn format_stream<R: Read, W: Write>(
    &mut self, 
    config: &C, 
    input: R, 
    output: W
) -> Result<(), Error>
```

#### c) Добавьте async поддержку:
```rust
pub async fn format_async(&mut self, config: &C, code: String) -> Result<String, Error>
```

### 6. **Конфигурация** ⚙️

**Проблемы:**
- Поддерживается только YAML
- Нет валидации конфигурации
- Нет способа переопределить конфигурацию через CLI

**Рекомендации:**
- Добавьте поддержку TOML, JSON
- Добавьте JSON Schema для конфигурации
- Реализуйте merge стратегию для конфигов (файл + CLI аргументы)

### 7. **Производительность** ⚡

**Рекомендации:**
- Добавьте кэширование разобранных деревьев
- Используйте `Cow<str>` вместо `String` где возможно
- Добавьте lazy parsing для больших файлов
- Используйте `SmallVec` для коллекций edits

### 8. **Расширяемость** 🔌

**Рекомендации:**

#### a) Добавьте plugin систему:
```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn init(&mut self, config: &Config) -> Result<(), Error>;
    fn pre_format(&self, state: &ParseState) -> Result<(), Error>;
    fn post_format(&self, state: &ParseState) -> Result<(), Error>;
}
```

#### b) Добавьте hooks:
```rust
pub trait FormatHook {
    fn on_file_start(&self, path: &Path);
    fn on_file_end(&self, path: &Path, changed: bool);
    fn on_error(&self, path: &Path, error: &Error);
}
```

### 9. **CLI улучшения** 💻

**Рекомендации:**
- Добавьте прогресс бар для обработки множества файлов
- Добавьте `--diff` mode для показа изменений
- Добавьте `--fix` flag для автоматического исправления
- Добавьте verbose/quiet режимы
- Добавьте цветной вывод с поддержкой NO_COLOR

### 10. **Cargo.toml улучшения** 📦

**Рекомендации:**
```toml
[package]
name = "fmt-runner"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <email@example.com>"]
license = "MIT OR Apache-2.0"
description = "A flexible code formatting framework based on Tree-sitter"
repository = "https://github.com/k1mb1/fmt-runner"
readme = "README.md"
keywords = ["formatter", "tree-sitter", "code-style"]
categories = ["development-tools", "parsing"]

[dependencies]
# Добавьте версии с минимальными фичами
tree-sitter = "0.25.9"
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.5", features = ["derive", "color"] }
serde_yaml = { version = "0.9", optional = true }
serde_json = { version = "1.0", optional = true }
toml = { version = "0.8", optional = true }
thiserror = "1.0"
log = "0.4"
env_logger = "0.11"
rayon = { version = "1.10", optional = true }  # для параллелизма
indicatif = { version = "0.17", optional = true }  # прогресс бары

[dev-dependencies]
tempfile = "3.0"
pretty_assertions = "1.0"
proptest = "1.0"

[features]
default = ["yaml", "json"]
yaml = ["dep:serde_yaml"]
json = ["dep:serde_json"]
toml = ["dep:toml"]
parallel = ["dep:rayon"]
progress = ["dep:indicatif"]
```

### 11. **Безопасность и стабильность** 🔒

**Рекомендации:**
- Добавьте обработку очень больших файлов (защита от OOM)
- Добавьте timeout для парсинга
- Валидируйте UTF-8 перед парсингом
- Добавьте recovery механизм для частично невалидного синтаксиса

### 12. **CI/CD** 🚀

**Рекомендации:**
- Создайте GitHub Actions workflow для:
  - Тестирования на разных платформах (Linux, macOS, Windows)
  - Проверки форматирования (`cargo fmt`)
  - Линтинга (`cargo clippy`)
  - Проверки документации (`cargo doc`)
  - Security audit (`cargo audit`)
- Настройте автоматический release процесс

### 13. **Дополнительные фичи** ✨

**Рекомендации:**
- Добавьте LSP (Language Server Protocol) поддержку
- Добавьте watch mode для автоматического форматирования при изменении файлов
- Добавьте git hooks интеграцию (pre-commit)
- Добавьте EditorConfig поддержку
- Создайте VS Code/IntelliJ плагины

## 📊 **Приоритизация**

### Высокий приоритет:
1. README.md и документация
2. Исправление InputEdit координат
3. Расширенное тестирование
4. Улучшенная обработка ошибок

### Средний приоритет:
5. Параллельная обработка файлов
6. CLI улучшения (прогресс, diff mode)
7. Поддержка множества форматов конфигурации
8. CI/CD setup

### Низкий приоритет:
9. Plugin система
10. Async API
11. LSP поддержка
12. IDE плагины

Библиотека имеет хорошую базовую архитектуру, но нуждается в полировке и расширении функционала для production use!