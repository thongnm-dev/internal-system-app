---
name: tauri-app-scaffold
description: Add a new feature domain to an existing Tauri 2 + Vue 3 + Rust/PostgreSQL app, following a layered architecture and copy-paste templates. Use when a POC/app already exists on this stack and needs a new feature wired in end-to-end, or when asked to standardize a project for future features.
---

# Tauri App Feature Templates

This skill assumes the target app **already exists** — a POC or a fuller app already scaffolded on the Tauri 2 + Vue 3 + Rust/PostgreSQL stack. It does not bootstrap a brand-new project from zero; its job is to add a new feature domain the same way every other feature in a well-structured app on this stack is added, using the same layering end to end (FE: types → IPC commands → composable → page; BE: models → database → service → command).

## When invoked

1. **Ask what's missing, don't assume.** At minimum you need:
   - The new feature's domain name (e.g. "invoice", "inventory") — drives every file, type, and function name below.
   - Whether it needs PostgreSQL + stored procedures, or is local-only (skip the database-layer samples and dual-registration steps if so).
   - Whether the target app already has an established page shape (e.g. a search-Fieldset + DataTable + Dialog CRUD page) to mirror instead of the minimal sample.
   - Which Rust module-wiring convention the app uses: plain `mod` files (`commands/mod.rs` → `pub mod <domain>_commands;`) or a `modules/*.rs` + `#[path]` indirection — the registration step differs.

2. **Read `ARCHITECTURE.md`** in this skill's directory for the conventions reference (directory layout, IPC pattern, layering rules, styling conventions, stored-procedure-only DB access). Treat it as the source of truth for structure; do not assume it matches any single app's own CLAUDE.md, which may describe that app's specifics.

3. **Copy the sample code straight out of the FEATURE_TEMPLATES files** (`references/FEATURE_TEMPLATES.md` is the index; `references/FEATURE_TEMPLATES_FE.md` and `references/FEATURE_TEMPLATES_BE.md` hold the actual code) — these are the single source of truth for what a new feature's files look like. Rename the fictitious `template`/`Template` domain to the real one throughout, adjust to the app's actual established page shape if it's richer than the minimal sample, then wire it in per the registration steps stated in each sample's header comment (module declarations, `invoke_handler`/generate_handler list, `commands/index.ts` re-export, router entry, dual SP registration).

4. **If asked to standardize the target app itself** for future features — not just add one now — generate the per-package `_template` starter files described in `references/FEATURE_TEMPLATES.md` directly inside the app's own real folders (e.g. `src/features/_template/`, `src-tauri/src/{models,database,services,commands}/_template*.rs`, `docs/store-procedure/_TEMPLATE.md`), plus a `docs/NEW_FEATURE_TEMPLATE.md` index. Base them on the app's actual established feature shape if it has one, not blindly on the minimal samples in `FEATURE_TEMPLATES_FE.md`/`_BE.md`.

## Output expectations

- Write real, wired-in code — deserialize → service → store → SP, not TODO-only stubs.
- If the app's real conventions have diverged from the samples (richer page shape, different module-wiring mechanism, different error type), follow the app's actual code, not the sample — the samples are a starting point for an app that has nothing yet, not an override for one that already has established patterns.
