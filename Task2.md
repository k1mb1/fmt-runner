k1mb1: Проанализируй fmt-runner и скажы Как улучшить эту библиотеку и в плане архитектуры и фич

GitHub Copilot: Я проанализирую структуру вашего проекта fmt-runner. Давайте начнем с изучения ключевых файлов. 

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/Cargo.toml)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/Tasks.md#1-1), lines 1 to 100

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/lib.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/core/engine.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/parser_core.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/pipeline_core.rs#1-1), lines 1 to 100

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/pass.rs#1-1), lines 1 to 100

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/builder.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/examples/cli_builder_usage.rs#1-1), lines 1 to 50

Searched text for `test|#\[test\]`, 20 results

Searched for files matching `**/*test*.rs`, no matches

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/handler.rs#1-1), lines 1 to 50



Отлично! Я провел детальный анализ вашей библиотеки **fmt-runner**. Это интересный проект для создания форматтеров кода на базе Tree-sitter. Вот подробные рекомендации по улучшению:

## 📊 **Анализ текущего состояния**

**Сильные стороны:**
- ✅ Хорошая модульная архитектура с разделением на core, parser, pipeline, cli
- ✅ Использование trait-based дизайна (Pass, StructuredPass, LanguageProvider)
- ✅ Builder pattern для CLI
- ✅ Type-safe подход с PhantomData

**Проблемные области:**
- ❌ Критическая ошибка в `apply_edit` - неправильное вычисление позиций для tree-sitter
- ❌ Минимальное тестовое покрытие
- ❌ Отсутствует документация (README.md)
- ❌ Использование `.expect()` вместо обработки ошибок
- ❌ Нет параллельной обработки файлов

---

## 🎯 **Рекомендации по улучшению**

### 1. **🔴 КРИТИЧЕСКАЯ ПРОБЛЕМА: InputEdit в parser_core.rs**

**Проблема:** В `apply_edit` вы используете байтовые позиции в качестве row/column для tree-sitter Point:

```rust
start_position: tree_sitter::Point {
    row: 0,
    column: start_byte,  // ❌ Неправильно!
},
```

**Решение:** Нужно конвертировать байтовые позиции в строку/столбец:

```rust
// Добавьте helper функцию
fn byte_to_point(source: &str, byte: usize) -> tree_sitter::Point {
    let before = &source[..byte.min(source.len())];
    let row = before.matches('\n').count();
    let column = before.split('\n').last().unwrap_or("").len();
    tree_sitter::Point { row, column }
}

// В apply_edit:
pub fn apply_edit(
    &mut self,
    state: &mut ParseState,
    start_byte: usize,
    old_end_byte: usize,
    new_text: &str,
) {
    let start_pos = Self::byte_to_point(&state.source, start_byte);
    let old_end_pos = Self::byte_to_point(&state.source, old_end_byte);
    
    state.source.replace_range(start_byte..old_end_byte, new_text);
    let new_end_pos = Self::byte_to_point(&state.source, start_byte + new_text.len());
    
    if let Some(tree) = &mut state.tree {
        let edit = InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte: start_byte + new_text.len(),
            start_position: start_pos,
            old_end_position: old_end_pos,
            new_end_position: new_end_pos,
        };
        tree.edit(&edit);
    }
    self.reparse(state);
}
```

---

### 2. **⚠️ Обработка ошибок**

**Проблемы:**
- Использование `.expect()` в parser_core.rs
- Недостаточная валидация

**Решения:**

```rust
// В Cargo.toml добавьте:
anyhow = "1.0"  // или используйте существующий thiserror

// Создайте src/error.rs:
#[derive(Debug, thiserror::Error)]
pub enum FmtRunnerError {
    #[error("Failed to load language grammar: {0}")]
    LanguageLoadError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid edit range: {start}..{end}")]
    InvalidEditRange { start: usize, end: usize },
}

// В Parser::new():
pub fn new() -> Result<Self, FmtRunnerError> {
    let mut ts_parser = TsParser::new();
    ts_parser
        .set_language(&Language::language())
        .map_err(|e| FmtRunnerError::LanguageLoadError(e.to_string()))?;
    
    Ok(Self {
        ts_parser,
        _marker: std::marker::PhantomData,
    })
}
```

---

### 3. **🚀 Производительность**

#### a) Параллельная обработка файлов

```rust
// В Cargo.toml:
rayon = "1.10"

// В engine.rs:
use rayon::prelude::*;

pub fn format_parallel(
    &mut self,
    config: &C,
    codes: &[String],
    files: &[PathBuf],
) -> Result<Vec<PathBuf>, std::io::Error> {
    use std::sync::Mutex;
    
    // Создаем парсеры для каждого потока
    let changed_files = Mutex::new(Vec::new());
    
    codes.par_iter()
        .zip(files.par_iter())
        .try_for_each(|(code, file)| -> Result<(), std::io::Error> {
            let mut local_parser = Parser::<Language>::new();
            let mut state = ParseState::new(code.clone());
            
            // Применяем pipeline локально
            for pass in self.pipeline.passes() {
                local_parser.parse(&mut state);
                let root = state.tree().unwrap().root_node();
                let edits = pass.run(config, &root, state.source());
                
                for edit in edits {
                    local_parser.apply_edit(&mut state, edit.range.0, edit.range.1, &edit.content);
                }
            }
            
            let formatted = state.source();
            if formatted != code {
                std::fs::write(file, formatted)?;
                changed_files.lock().unwrap().push(file.clone());
            }
            Ok(())
        })?;
    
    Ok(changed_files.into_inner().unwrap())
}
```

#### b) Кэширование результатов парсинга

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct CachedParser<Language: LanguageProvider> {
    parser: Parser<Language>,
    cache: HashMap<u64, tree_sitter::Tree>,
}

impl<Language: LanguageProvider> CachedParser<Language> {
    pub fn parse_cached(&mut self, state: &mut ParseState) {
        let hash = self.hash_source(state.source());
        
        if let Some(cached_tree) = self.cache.get(&hash) {
            state.tree = Some(cached_tree.clone());
        } else {
            self.parser.parse(state);
            if let Some(ref tree) = state.tree {
                self.cache.insert(hash, tree.clone());
            }
        }
    }
    
    fn hash_source(&self, source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }
}
```

---

### 4. **📚 Документация**

**Создайте README.md:**

```markdown
# fmt-runner

A powerful, extensible code formatter framework built on Tree-sitter.

## Features

- 🌳 **Tree-sitter based** - Fast, reliable parsing
- 🔌 **Plugin architecture** - Easy to add custom formatting passes
- ⚡ **Parallel processing** - Format multiple files simultaneously
- 🎯 **Type-safe** - Leverages Rust's type system
- 🛠️ **CLI ready** - Built-in command-line interface

## Quick Start

\`\`\`rust
use fmt_runner::{cli_builder, Pass, LanguageProvider};

// Define your language
struct MyLanguage;
impl LanguageProvider for MyLanguage { /* ... */ }

// Create custom pass
struct MyPass;
impl Pass for MyPass { /* ... */ }

fn main() {
    cli_builder::<MyLanguage, MyConfig>()
        .add_pass(MyPass)
        .run();
}
\`\`\`

## Documentation

See [docs.rs](https://docs.rs/fmt-runner) for full API documentation.
```

---

### 5. **🧪 Тестирование**

**Создайте tests/integration_test.rs:**

```rust
use fmt_runner::*;

#[test]
fn test_basic_formatting() {
    let source = "fn main(){println!(\"hello\");}";
    let mut state = ParseState::new(source.to_string());
    
    let mut pipeline = Pipeline::new();
    pipeline.add_pass(SpacingPass);
    
    let mut engine = Engine::<RustLanguage, Config>::new(pipeline);
    // ... test formatting
}

#[test]
fn test_idempotency() {
    // Форматирование дважды должно давать одинаковый результат
}

#[test]
fn test_invalid_syntax() {
    // Должен обрабатывать невалидный синтаксис
}
```

**Property-based тесты:**

```rust
// В Cargo.toml:
[dev-dependencies]
proptest = "1.4"

// В tests/properties.rs:
use proptest::prelude::*;

proptest! {
    #[test]
    fn formatting_is_idempotent(code in ".*") {
        let result1 = format_code(&code);
        let result2 = format_code(&result1);
        prop_assert_eq!(result1, result2);
    }
    
    #[test]
    fn formatting_preserves_semantics(code in valid_code_generator()) {
        let formatted = format_code(&code);
        prop_assert!(semantically_equivalent(&code, &formatted));
    }
}
```

---

### 6. **🏗️ Архитектурные улучшения**

#### a) Добавьте Builder для Engine

```rust
pub struct EngineBuilder<Language: LanguageProvider, Config> {
    pipeline: Pipeline<Config>,
    cache_enabled: bool,
    parallel: bool,
    _marker: PhantomData<Language>,
}

impl<Language: LanguageProvider, C> Engine<Language, C> {
    pub fn builder() -> EngineBuilder<Language, C> {
        EngineBuilder {
            pipeline: Pipeline::new(),
            cache_enabled: false,
            parallel: false,
            _marker: PhantomData,
        }
    }
}

impl<Language: LanguageProvider, C> EngineBuilder<Language, C> {
    pub fn with_caching(mut self) -> Self {
        self.cache_enabled = true;
        self
    }
    
    pub fn with_parallel(mut self) -> Self {
        self.parallel = true;
        self
    }
    
    pub fn build(self) -> Engine<Language, C> {
        // создать engine с нужными настройками
    }
}
```

#### b) Добавьте поддержку конфигурации через файл

```rust
// В Cargo.toml добавьте:
toml = "0.8"

// src/config/mod.rs:
pub trait ConfigLoader {
    fn load_from_file(path: &Path) -> Result<Self, ConfigError>
    where
        Self: Sized;
    
    fn save_to_file(&self, path: &Path) -> Result<(), ConfigError>;
}

impl<T: Serialize + DeserializeOwned> ConfigLoader for T {
    fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(Into::into)
    }
    
    fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

#### c) Добавьте Middleware/Hook систему

```rust
pub trait Hook<Config> {
    fn before_pass(&self, config: &Config, node: &Node, source: &str);
    fn after_pass(&self, config: &Config, edits: &[Edit]);
}

pub struct Pipeline<Config> {
    passes: Vec<Box<dyn ErasedPass<Config>>>,
    hooks: Vec<Box<dyn Hook<Config>>>,
}

impl<Config> Pipeline<Config> {
    pub fn add_hook<H: Hook<Config> + 'static>(&mut self, hook: H) {
        self.hooks.push(Box::new(hook));
    }
}
```

---

### 7. **✨ Новые фичи**

#### a) Streaming API для больших файлов

```rust
use std::io::{BufRead, BufReader, Write};

pub fn format_stream<R: BufRead, W: Write>(
    &mut self,
    config: &C,
    reader: R,
    mut writer: W,
) -> Result<(), std::io::Error> {
    let mut buffer = String::new();
    let mut reader = reader;
    
    while reader.read_line(&mut buffer)? > 0 {
        let mut state = ParseState::new(buffer.clone());
        self.run(config, &mut state);
        writer.write_all(state.source().as_bytes())?;
        buffer.clear();
    }
    
    Ok(())
}
```

#### b) Поддержка incremental formatting

```rust
pub struct IncrementalEngine<Language: LanguageProvider, Config> {
    engine: Engine<Language, Config>,
    last_state: Option<ParseState>,
}

impl<Language: LanguageProvider, C> IncrementalEngine<Language, C> {
    pub fn format_incremental(
        &mut self,
        config: &C,
        changed_range: (usize, usize),
    ) -> String {
        // Перефоматировать только измененную часть
    }
}
```

#### c) LSP поддержка

```rust
// Новый модуль src/lsp/mod.rs
use tower_lsp::{LspService, Server};

pub async fn run_lsp_server() {
    let (service, socket) = LspService::new(|client| {
        FmtRunnerLanguageServer::new(client)
    });
    
    Server::new(stdin(), stdout(), socket).serve(service).await;
}
```

---

### 8. **📦 CI/CD и инфраструктура**

**Создайте .github/workflows/ci.yml:**

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/tarpaulin@v0.1
      - uses: codecov/codecov-action@v3
```

---

## 📋 **Приоритеты внедрения**

### Критично (сделать первым):
1. ✅ Исправить `apply_edit` - конвертация byte → Point
2. ✅ Заменить `.expect()` на правильную обработку ошибок
3. ✅ Добавить README.md с примерами

### Важно:
4. ✅ Добавить unit-тесты для всех модулей
5. ✅ Добавить параллельную обработку файлов
6. ✅ Добавить CI/CD pipeline

### Желательно:
7. ✅ Добавить кэширование
8. ✅ Расширить Builder API
9. ✅ Добавить property-based тесты
10. ✅ Поддержка конфигурации через файлы

### В будущем:
11. ✅ LSP поддержка
12. ✅ Streaming API
13. ✅ Incremental formatting
14. ✅ Benchmark suite

---

## 🎓 **Заключение**

Ваша библиотека имеет **солидную основу**, но нуждается в:
- **Исправлении критических багов** (InputEdit)
- **Улучшении тестового покрытия**
- **Добавлении документации**
- **Оптимизации производительности**

Следуя этим рекомендациям, fmt-runner может стать **мощным и надежным** фреймворком для создания форматтеров!