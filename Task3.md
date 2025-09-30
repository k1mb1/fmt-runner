k1mb1: Проанализируй fmt-runner и скажы Как улучшить эту библиотеку и в плане архитектуры и фич

GitHub Copilot: Gathering key source files to understand current architecture for targeted improvement recommendations. 

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/lib.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/builder.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/cli_entry.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/commands/format.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/core/engine.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/language_provider.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/parse_state.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/parser_core.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/edit.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/pass.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/pipeline_core.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/commands/utils.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/supported_extension/mod.rs)

Дал обзор текущей архитектуры и подготовил набор улучшений: от быстрых "quick wins" до более серьёзных архитектурных изменений и идей для фич. Ниже структурировано, чтобы можно было выбрать приоритеты.

## Общая оценка

Библиотека уже аккуратно разделена на слои:
- cli (парсинг аргументов, загрузка конфига)
- core (Engine)
- parser (Tree‑sitter parser + состояние)
- pipeline (Pass / StructuredPass / Pipeline / Edit)
- supported_extension (утилита расширений)

Плюсы:
- Простая и понятная точка входа (`Engine`, `Pipeline`, `Pass`)
- Разделение CLI и ядра (хотя можно усилить)
- Унифицированный `Config` тип
- Возможность расширять язык через `LanguageProvider`
- Есть тесты на базовые структуры

Главные текущие ограничения:
1. Инкрементальное редактирование реализовано формально: `InputEdit` заполняется с координатами `row=0`, что неверно для многострочных правок → потенциально некорректное или неоптимальное обновление дерева.
2. Каждый pass генерирует edits, которые применяются сразу и заново парсятся после КАЖДОГО edit — это дорого при большом файле.
3. Нет защиты от пересекающихся/конфликтующих диапазонов правок.
4. Нет способа работать с несколькими языками в одном запуске (один `Language` на весь `Engine`).
5. Отсутствует система плагинов / внешних расширений.
6. Конфиг глобальный для всех pass — нет per-pass overrides, профилей или слоёв (user/project/inline).
7. Нет LSP / диапазонного форматирования / diff‑режима / watch‑режима.
8. Ошибки из pass только печатаются через `eprintln!`, нет унифицированного репортинга.
9. Тестовое покрытие минимально: нет тестов на взаимную композицию pass, на стабильность дерева, на конфликт редактирования.
10. Нет трейсинга производительности, метрик, профилирования.
11. API не возвращает структуру результата (только векторы изменённых путей).

## Архитектурные улучшения

### 1. Модель применения правок
Сейчас: для каждого pass → edits → сортировка → последовательное применение → инкрементальный reparse.
Проблемы:
- Неверные позиции в `InputEdit`
- Многократный reparse при большом количестве edits

Предложения:
- Собрать все edits всех pass в один буфер, сначала проверить конфликты, потом применить батчом (обратный порядок), затем один полный parse.
- Либо двухфазная схема:
  1. Анализ (каждый pass возвращает edits)
  2. Применение / агрегирование / нормализация (merge смежных, detect overlap)
  3. Один reparse
- Для реально инкрементальных сценариев — ввести line map + вычисление `row/column` по байтам (кэшировать массив накопленных смещений по строкам).

### 2. Контекст для pass
Сделать `FormatterContext`:
- Доступ к: `source`, `tree`, helper-и (поиск узлов, диапазоны)
- Возможность копить предупреждения / diagnostics
- Кэш узлов (чтобы не пересчитывать выборки)
- Хранение shared state между pass (например, список импортов)

Интерфейс:
```
fn run(&self, ctx: &mut FormatterContext<Self::Config>) -> Vec<Edit>;
```

### 3. Pass зависимости и порядок
- Добавить optional metadata: `fn name()`, `fn phase()`, `fn depends_on() -> &'static [&'static str]`
- Возможность топологически выстраивать pipeline автоматически
- Разделение фаз: нормализация → группировка → раскладка → финальный вывод (как в Rustfmt)

### 4. Мульти-языковость
Текущий `Engine<Language, Config>` жёстко параметризован. Идеи:
- Ввести `LanguageRegistry`: маппинг расширений → `Box<dyn LanguageAdapter>`
- `Engine` принимает набор файлов разных языков, группирует по языку, каждый блок обрабатывает своим под‑движком.
- Унифицировать `ParseState` → хранить `language_id`.

### 5. Улучшение `ParseState`
- Добавить line index (Vec<usize>) для быстрого вычисления `(row, column)`
- Метод `byte_to_point` / `range_to_points`
- Ленивая пересборка индекса при изменении
- Возможность хранить "snapshots" (полезно для diff)

### 6. Обработка конфликтов правок
- Функция `normalize_edits(Vec<Edit>) -> Result<Vec<Edit>, EditConflict>`
- Детект пересечений и сортировка+слияние (если совместимо)
- Возврат diagnostics с указанием pass, которые конфликтуют

### 7. Конфигурация
Расширить модель:
- Слои: built‑in defaults → глобальный (XDG) → проектный → локальный (переданный пользователем) → inline overrides / flags
- Per-pass секция:
```
[pass.imports]
reorder = true
group_blank_lines = 1
```
- Профили:
```
profile = "strict"
[profiles.strict]
max_line_length = 80
```

### 8. Ошибки и диагностика
Создать доменную модель:
```
pub enum FormatDiagnosticKind { Warning, Error, Info }
pub struct Diagnostic { span: (usize, usize), message: String, pass: String, kind: FormatDiagnosticKind }
```
Engine возвращает `FormatResult { formatted: String, diagnostics: Vec<Diagnostic>, changed: bool }`

### 9. Tracing / observability
- Использовать `tracing` вместо (или вместе) `log`
- Span на каждый pass, метрики: время, число edits
- Feature flag: `metrics` (включает, например, `prometheus` экспортер для daemon режима)

### 10. Взаимодействие с CLI
Отделить бинарь в отдельном крейте (`fmt-runner-cli`) и оставить эту библиотеку как `fmt-runner-core`
- Feature `cli` в текущем крейте для опционального включения зависимостей (clap, serde_yaml, env_logger)
- Это упростит реюз в других приложениях / IDE интеграциях

### 11. Расширяемость / плагины
Варианты:
- Dynamic load через `libloading` (риск + сложность ABI)
- Регистрация через функцию:
```
register_pass!(MyPass);
```
- Макро для декларации pass c metadata

### 12. Диапазонное форматирование
Добавить API:
```
engine.format_range(config, file, range: (usize, usize))
```
— Поддержка LSP `textDocument/rangeFormatting`, `onTypeFormatting`.

### 13. Интеграция с LSP
Вынести адаптер:
- LSP server (tower-lsp)
- Поддержка incremental text sync
- Кэш parse trees между запросами

### 14. Отложенное применение правок
Опция: возвращать diff (unified diff) без записи
→ Удобно для CI:
```
--mode diff
```

### 15. Производительность / параллелизм
- Параллельная обработка файлов (Rayon) когда нет межфайловых зависимостей
- Ограничить размер edits на файл (fail‑fast)
- Memory arena для временных строк в pass

### 16. Безопасность к UTF‑8 / границам
Сейчас диапазоны — байтовые. Нужно гарантировать, что не режем внутри code point. Добавить helper `safe_slice(range)` + в тестах property‑based (proptest) генерировать Unicode.

### 17. Тестовое покрытие
Добавить:
- Snapshot тесты (insta) для format результатов
- Conflict тесты (два pass намеренно генерируют пересекающиеся edits)
- Fuzz: рандомно генерируем исходник, применяем identity pass → assert no change
- Инварианты: после всех edits дерево успешно парсится

### 18. Улучшение API Engine
Сейчас:
```
check(&config, codes, files) -> Vec<PathBuf>
format_and_write(...) -> Result<Vec<PathBuf>, _>
```
Предложение:
```
pub struct FileFormatOutcome {
    pub path: PathBuf,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn format_batch(&mut self, batch: &[FileInput], opts: &FormatOptions) -> BatchResult
```
Где `BatchResult` содержит агрегированные метрики.

### 19. Кодогенерация / вспомогательные макросы
Макро для `LanguageProvider`:
```
language_provider!(RustLanguage, tree_sitter_rust::language(), ["rs"]);
```

### 20. Структура проекта
Рефакторинг:
```
fmt-runner/
  crates/
    core/
    cli/ (зависит от core)
    lsp/ (опционально)
```
В Cargo.toml workspaces + feature gating.

## Feature Roadmap (пример приоритезации)

Quick wins (день-два):
- Исправить расчёт `InputEdit` точек (row/column)
- Добавить проверку пересечений edits
- Вынести диагностику из `eprintln!` в структуру
- Feature `cli` + отделить зависимости
- Snapshot тест на стабильность форматирования

Среднесрочно:
- Батч-применение edits
- Мульти-языковая поддержка через реестр
- Range formatting API
- Diff режим
- Pass metadata + автоматическая сортировка

Долгосрочно:
- LSP сервер
- Плагины
- Параллелизм + кэш AST
- Конфигурационные слои и профили
- Профилирование + metrics endpoint

## Пример улучшенного фрагмента (идея)

Инкапсуляция применения edits:
```
pub fn apply_all(state: &mut ParseState, edits: &mut [Edit]) {
    edits.sort_by(|a,b| b.range.0.cmp(&a.range.0));
    for e in edits {
        // безопасное применение + вычисление Point
        apply_single(state, e);
    }
    reparsed_full(state);
}
```

Расчёт позиций:
```
fn byte_to_point(line_index: &[usize], byte: usize) -> Point {
    let line = match line_index.binary_search(&byte) {
        Ok(l) => l,
        Err(ins) => ins - 1,
    };
    Point { row: line, column: byte - line_index[line] }
}
```

## Потенциальные риски и внимание

- Добавление батчевой схемы может осложнить debug — стоит предусмотреть режим "step-by-step".
- Диагностика и метаданные увеличат размер API — нужно думать о backward compatibility.
- Мульти-языковость + registry — усложнит generic API; можно начать с enum или trait objects.
- Плагины через динамические библиотеки → ABI/безопасность; лучше начать с compile-time регистрации.

## Рекомендованные инструменты/крейты

- tracing / tracing-subscriber (вместо log)
- rayon (параллельная обработка)
- insta (snapshot тесты)
- anyhow + thiserror (единая ошибка)
- line-index (есть готовые реализации, но можно и свою)
- tower-lsp (LSP)

## Резюме

Основные точки роста: корректная и эффективная система применения правок, расширяемость (плагины, мульти-языки), улучшенная конфигурация и диагностика, подготовка к LSP и range formatting. Начните с исправления `InputEdit` и батчевого применения edits — это фундамент для дальнейших улучшений производительности и корректности.

Если хочешь — могу следующим шагом:
1. Написать патч для корректного вычисления `row/column`.
2. Добавить проверку пересечений edits.
3. Ввести структуру `Diagnostic`.

Просто скажи, с чего начать.