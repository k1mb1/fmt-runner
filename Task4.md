k1mb1: Проанализируй fmt-runner и скажы Как улучшить эту библиотеку и в плане архитектуры и фич

GitHub Copilot: Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/lib.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/core/engine.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/edit.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/pipeline_core.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/pipeline/pass.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/parser_core.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/parse_state.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/language_provider.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/supported_extension/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/builder.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/handler.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/cli_entry.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/commands/init.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/commands/format.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/commands/utils.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/error.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/cli/commands/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/parser/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/src/core/mod.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/examples/cli_builder_usage.rs)

Read [](file:///home/k1mb/RustroverProjects/fmt-runner/Task3.md)

Reviewing fmt-runner now to outline its current design, pinpoint architectural pain points, and suggest feature upgrades.

## Game plan
- Capture the core architecture as it stands and highlight notable strengths/weaknesses.
- Recommend structural upgrades that improve correctness, extensibility, and performance.
- Suggest user-facing features and tooling that unlock new workflows or DX wins.
- Flag complementary testing/observability work that de-risks those changes.

## Snapshot of today’s architecture
- Clean separation of concerns: `cli` (argument parsing/config IO), `core::Engine` (orchestrates parse + passes), `parser` (Tree-sitter integration), `pipeline` (passes/edit abstractions), `supported_extension`.
- Config type is generic and consistent across passes, CLI, and engine.
- `LanguageProvider` trait makes languages pluggable, but only one language per engine run.
- `Engine::run` parses once per file, then each pass emits edits applied immediately via incremental reparsing.

## Architectural upgrades worth tackling
- **Correct InputEdit coordinates:** today every edit uses `row=0`, `column=start_byte`, which breaks for multi-line edits. Introduce a line-index cache in `ParseState` to map bytes to `Point`, and reuse it for all edits.
- **Batch edit application:** gather edits from all passes, detect overlaps/conflicts, merge adjacent ranges, then apply once before a single reparse. Big perf + correctness boost.
- **Edit normalization/diagnostics:** return a result such as `Result<Vec<Edit>, EditConflict>` so conflicting passes surface clear errors instead of silently clobbering each other.
- **Formatter context object:** replace direct `(root, source)` passing with `FormatterContext` that exposes cached AST queries, shared state, and a diagnostics sink, reducing duplication and improving pass ergonomics.
- **Pass metadata & dependency graph:** optional `fn name()`, `fn phase()`, `fn depends_on()` allow automatic ordering/topological sort and phase-based execution (normalize, reorder, layout, etc.).
- **Config layering:** support layered configs (defaults → global → project → CLI override) and per-pass sections (`[pass.imports]`). Add profile selection (`profile = "strict"`).
- **Multi-language registry:** let the engine accept multiple `LanguageProvider`s, bucket files by extension, and run language-specific pipelines; useful for polyglot repos.
- **Richer Engine API:** instead of just changed-path vectors, return `FileFormatOutcome { changed, diagnostics, duration }` and an aggregate `BatchResult` for better automation hooks.
- **Structured diagnostics:** standardize warnings/errors from passes with spans, severities, and pass names; surface them in CLI output and potential IDE integrations.
- **Observability:** switch to `tracing` spans and optionally collect metrics (time per pass, edit counts). Provide a feature flag for production logging vs silent library mode.

## Feature expansions to consider
- **Diff/CI mode:** new `FormatMode::Diff` that prints unified diffs without writing files, ideal for CI checks.
- **Range & on-type formatting:** expose `format_range`/`format_snippet` APIs, groundwork for editors.
- **LSP server crate:** build a `tower-lsp` powered daemon using the shared engine, supporting incremental document sync and diagnostics streaming.
- **Watch mode/daemon:** CLI flag to watch directories and auto-format on change with debounced runs.
- **Plugin story:** compile-time registration macros or dynamic loading for third-party passes (start with macro-based registry to avoid ABI headaches).
- **Parallel formatting:** integrate Rayon to process independent files concurrently once the edit pipeline is deterministic.
- **UTF-8 safety helpers:** utility to ensure edit ranges align to char boundaries, plus property-based tests using `proptest`.

## Testing & tooling boosts
- Expand coverage with snapshot tests (`insta`) verifying end-to-end formatting stability.
- Conflict regression tests ensuring overlapping-edit detection works.
- Performance regression benchmarks (criterion) once batching lands.
- Fuzzing harness that ensures formatting doesn’t panic and re-parsed trees remain valid.
- Optional `cargo fmt-runner` subcommand for template projects, bundling default passes/config.

## Suggested implementation order
1. Fix `InputEdit` coordinate calculation with a reusable line index + add unit tests.
2. Introduce edit normalization/batching and return conflict diagnostics.
3. Add structured diagnostics to the `Engine` result and thread them through CLI output.
4. Split the crate into workspace members (`fmt-runner-core`, `fmt-runner-cli`) gated by features to trim dependencies for library consumers.
5. Layered config and per-pass metadata (unlocks more advanced passes).
6. Explore range formatting + diff mode; then tackle LSP/daemon once API stabilizes.

## Quality gates
- Build: not run (analysis only).
- Tests: not run (analysis only).

**Requirements coverage:** Architectural recommendations — Done; Feature ideas — Done.

Happy to dive into any of these changes next—just point me at the one you want implemented first.