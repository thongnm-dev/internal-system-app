# Feature templates — standardizing how new features get added

This skill assumes the target app already exists (a POC or a fuller app already scaffolded on the Tauri 2 + Vue 3 + Rust/PostgreSQL stack) — there is no separate "initial scaffold" step or starter app bundled with this skill. `ARCHITECTURE.md` is the conventions reference; this file (and its two companions) is the ready-to-copy code for adding the app's 2nd, 5th, 20th feature domain, whenever that need comes up — including the very first time this skill is invoked for a given app.

The pattern: drop one throwaway, unregistered starter file into **each package/layer** the new app has, named after a fictitious `template`/`Template` domain. Each one's header comment states the rule for that layer; the code below it is a working example of following that rule. Because it's real code sitting in the real folder — not a separate doc that drifts out of sync — it stays accurate for as long as the layer's actual pattern doesn't change, and anyone (or any agent) adding a feature can `grep` for `_template` and get a concrete, current answer instead of half-remembering a convention from a much earlier conversation.

Applying this pattern to an app looks like: `src/features/_template/`, `src/tauri/commands/_template.ts`, `src-tauri/src/{models,database,services,commands}/_template*.rs`, `docs/store-procedure/_TEMPLATE.md`, and an index at `docs/NEW_FEATURE_TEMPLATE.md`.

Ready-to-copy code samples, split by side:
- **[FEATURE_TEMPLATES_FE.md](FEATURE_TEMPLATES_FE.md)** — types, IPC commands, composable, page.
- **[FEATURE_TEMPLATES_BE.md](FEATURE_TEMPLATES_BE.md)** — models, database store, service, command, stored procedures, schema.

Two things are genuinely project-specific and won't match the samples out of the box:
- **Rust module wiring.** The BE samples use plain `mod` declarations (`commands/mod.rs` → `pub mod template_commands;`, etc.) — the simplest, most common mechanism. If the target app instead uses a `modules/*.rs` + `#[path]` indirection, register there instead — same rule, different file.
- **Page/composable richness.** The FE page sample is the minimal shape (list + inline add). If the app has already standardized on something richer (e.g. a search-Fieldset + DataTable + Dialog CRUD page), rewrite the sample to match *that* shape instead — the template must mirror what the app actually does elsewhere, not what this skill guessed on day one.

## Index file — `docs/NEW_FEATURE_TEMPLATE.md`

Generate one for the target app too: a table listing every template file (both FE and BE), its rename target, and every place it must be registered — so the "where do I wire this in" answer is one file away instead of re-derived from scratch each time.

## Making a template file actually unregistered

Whatever mechanism the new app uses to wire modules together, the template must sit outside it, so it never compiles/builds/routes as live code:

- **TypeScript**: don't add it to `commands/index.ts`'s re-exports; nothing imports it, so it's dead code that still type-checks.
- **Vue**: don't add a route for it in `routes.ts`.
- **Rust**: don't declare it with `mod`/`pub mod` anywhere reachable from `lib.rs`.
- **SQL**: keep it as a fenced code block inside a `.md` file, not a loose `.sql` file in `docs/store-procedure/` — a stray real `.sql` file there could get mistaken for a procedure someone forgot to register.

## When to generate the per-package `_template` starter files (vs. just adding one feature)

These are two different asks:
- **Add this one feature now** → just copy the FE/BE samples, rename the domain, wire it in. No `_template` files needed.
- **Standardize the app so future features follow this shape too** → additionally generate the per-package `_template` starter files (unregistered, copy-paste-ready, one per layer) directly in the app's real folders, plus a `docs/NEW_FEATURE_TEMPLATE.md` index — do this once the app has at least one real feature domain built (so the `_template` files mirror its actual established shape, not a guess), or whenever the user explicitly asks to standardize/template the project.

If standardization is asked for before any real feature exists yet, the samples in the two companion files are the fallback baseline for the `_template` files too — just say so, and expect them to need a refresh once the app's real UI/CRUD shape diverges from that starting point.
