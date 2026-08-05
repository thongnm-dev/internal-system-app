# Architecture Template — Tauri 2 + Vue 3 + Rust/PostgreSQL

Generic reference architecture for a desktop app on this stack. Use it as a starting point for a **new, unrelated** app on the same stack — copy the structure, not the business domains. Anything under "Example domains" is illustrative only.

## Tech Stack

- **Frontend:** Vue 3 (Composition API, `<script setup lang="ts">`), Vite, TypeScript, Tailwind CSS, PrimeVue 4
- **State:** Pinia stores for cross-cutting app state (auth, nav/menu config, shared reference data); Vue composables for feature-level state
- **Routing:** Vue Router with `createWebHashHistory()` (required for Tauri's file:// production build — history mode breaks on reload)
- **Desktop shell:** Tauri v2 with Rust backend
- **Database (if needed):** PostgreSQL, accessed **exclusively** through stored procedures — no raw SQL in the app layer
- **Common Tauri plugins:** `dialog`, `updater`, `process`, `notification` (add only what the app needs)

Drop PostgreSQL/stored-procedures entirely if the new app is local-only (SQLite, flat files, or no persistence) — everything below still applies except the `database/` layer and the SP conventions.

## Repo Layout

```
<app-root>/
├─ src/                        # Vue frontend
│  ├─ app/
│  │  ├─ router/               # Vue Router setup + route definitions
│  │  ├─ stores/                # Pinia stores (auth, menu/nav, shared reference data)
│  │  └─ plugins/               # PrimeVue + Pinia registration
│  ├─ features/                 # One folder per business domain
│  │  └─ <domain>/
│  │     ├─ components/         # Vue SFC pages/widgets for this domain
│  │     ├─ composables/        # use<Domain>() — owns state, data fetching, Tauri calls
│  │     └─ utils/              # (optional) domain-specific helpers
│  ├─ shared/
│  │  ├─ components/            # App shell pieces: sidebar, header, bottom bar, loading/toast, error screens
│  │  ├─ composables/           # Cross-cutting composables (app shell, updater, network/db status, toast)
│  │  ├─ utils/                 # Generic utilities with no domain ownership
│  │  └─ config/                # App config, theme tokens
│  ├─ _/
│  │  └─ types/                 # Shared TypeScript types, one file per domain
│  ├─ tauri/
│  │  ├─ commands/              # IPC wrappers: _base.ts + one file per backend domain
│  │  └─ events.ts              # Listeners for backend-pushed Tauri events
│  ├─ App.vue                   # Shell: sidebar + header + router-view + bottom bar
│  ├─ main.ts
│  └─ styles.css                # Tailwind + CSS-variable theming + component overrides
├─ src-tauri/                   # Rust backend
│  ├─ src/
│  │  ├─ commands/              # #[tauri::command] handlers — thin, delegate to services
│  │  ├─ services/               # Business logic, one service per domain
│  │  ├─ database/               # PostgreSQL store layer (calls stored procedures only)
│  │  ├─ models/                 # Request/response DTOs, one file per domain
│  │  ├─ utils/                  # api_client, time, app_config, db connect, network, email, etc.
│  │  ├─ app/                    # Error type, Result alias, shared consts
│  │  └─ lib.rs                  # Command registration, plugin setup, background services
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ docs/
│  └─ store-procedure/           # SQL source for every stored procedure (if using Postgres)
├─ package.json
└─ (skill file, if reused) CLAUDE.md
```

## Frontend Conventions

**Tauri IPC pattern** — all backend calls funnel through `src/tauri/commands/`:
- `_base.ts` exports `safeInvoke<T>()` (wraps `invoke()`, normalizes errors), `canUseTauriRuntime()` (guards against calling Tauri APIs when running as a plain web page, e.g. during `npm run dev`), and `friendlyError()` (maps backend error codes to user-facing messages).
- One command file per backend domain, mirroring `src-tauri/src/commands/<domain>.rs` 1:1.
- `index.ts` re-exports everything, so features import from `@/tauri/commands` rather than reaching into individual files.
- `src/tauri/events.ts` centralizes listeners for events the Rust side pushes (background poll results, file-watcher notifications, etc.) — don't scatter `listen()` calls across components.

**Feature folder pattern** — every entry under `src/features/` follows:
```
<domain>/
├─ components/     # Pages, wired into router/routes.ts
├─ composables/    # use<Domain>() owns all state + Tauri calls for this domain
└─ utils/          # optional, domain-local helpers only
```
A component should not call `safeInvoke` directly — it calls the domain composable, which owns the IPC calls, error handling, and reactive state.

**Routing** — hash history, route table in `src/app/router/routes.ts`, auth/role guards in `src/app/router/index.ts`. Protected routes redirect to `/login` carrying a `returnPath` for post-login redirect.

**Styling** — Tailwind with CSS-variable-based theming, not raw palette colors. Define semantic tokens (e.g. `bg-canvas`, `text-ink`) in `styles.css`, map them in `shared/config/themeTokens.ts`, reference them in `tailwind.config.js`. Dark mode via a `[data-theme='dark']` selector, so components never branch on JS for color — only for structural changes.

**Form controls** — use PrimeVue components exclusively (`Checkbox`, `InputText`, `Select`, etc.), never raw `<input>`/`<select>`. The one allowed exception is a genuinely read-only, non-interactive display value (e.g. showing a picked file path) — use `<input readonly>` there, never for anything editable or checkable.

**Window sizing** — decide a default and minimum size up front (e.g. 1200×760 default / 980×600 minimum) and design every screen to fit the minimum without horizontal scroll.

## Backend Conventions

**Layering is one-directional:** `commands → services → database → utils`. A command handler never talks to the database layer directly, and a service never touches Tauri APIs (`AppHandle`, event emission) — pass what's needed as parameters or return values so services stay testable independent of Tauri.

- `commands/` — thin `#[tauri::command]` functions: deserialize input, call one service method, map `Result<T, AppError>` into whatever the frontend expects. No business logic here.
- `services/` — one file per domain, owns business logic and orchestration across multiple database calls if needed.
- `database/` — one store per domain. **If using PostgreSQL: every query is a call to a stored procedure — never inline `SELECT`/`INSERT`/`UPDATE`/`DELETE`.** This is what makes schema changes reviewable as SQL diffs and keeps privilege/audit logic in one place (the DB).
- `models/` — request/response DTOs (`Serialize`/`Deserialize` structs) shared between `commands` and `services`, one file per domain, mirroring the frontend's `src/_/types/`.
- `utils/` — connection pooling, config loading, external API clients, email, etc. — no business rules.
- `app/` — a shared `AppError` enum and `Result<T>` alias used across all layers, so error handling is consistent from database up to the command boundary.

**Stored procedure workflow (PostgreSQL only):**
1. Write the SQL in `docs/store-procedure/sp_<domain>_<action>.sql`.
2. Register it in **both**: the SP-management service's `all_procedures()` listing (so it shows up in any admin/ops UI) and the startup store's `ensure_stored_procedures()` (so it's installed/updated automatically on app start in dev). Forgetting either half is the most common way a new SP silently doesn't apply or doesn't show in the management screen.

**Background services** — long-running polling/watching tasks (file watchers, remote-API pollers) are started once in `lib.rs`'s setup hook and emit Tauri events on completion; they should not block app startup and should degrade gracefully (log + retry, don't panic) if the resource they watch is temporarily unavailable.

**Database initialization** — a `database/startup_store.rs::init()` that creates tables and installs/updates stored procedures, gated to run automatically only in debug builds; production installs are a deliberate, separate step.

## Example domains (illustrative only — do not copy into a new app)

A typical app built on this pattern accumulates domains like project/task management, daily notes & reports, an issue backlog with an external API integration, cloud storage browsing, usage tracking, and a governance area (users/roles/permissions/menus). None of that is part of this template — those are just examples of what fills in per app; replace with the new app's actual feature list.

## What to decide before scaffolding a new app

- App name, window title, default/minimum window size.
- Feature domain list (drives `src/features/*` and `src-tauri/src/commands/*` module names).
- Persistence: PostgreSQL + stored procedures, a lighter local store (SQLite), or none.
- Auth model: does it need login/roles at all, or is it single-user/local?
- Which Tauri plugins are actually needed (don't default to the full plugin list above).
