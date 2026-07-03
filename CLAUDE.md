# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Manager System Helps** — a Tauri 2.0 desktop application for internal project management and statistics tracking. The app imports CSV data, generates reports, tracks daily work, and converts Excel specs to Markdown.

## Commands

```bash
# Full development (Tauri + Vue hot reload)
npm run tauri:dev

# Frontend-only dev server (http://127.0.0.1:1420)
npm run dev

# Production desktop build
npm run tauri:build

# Frontend build only
npm run build

# Type-check Vue files
npx vue-tsc --noEmit
```

There are no test commands — the project has no test suite currently.

When developing the Rust backend, run Cargo commands from `src-tauri/`:
```bash
cd src-tauri && cargo check
cd src-tauri && cargo build
```

## Architecture

### Tech Stack

- **Frontend:** Vue 3 (Composition API with `<script setup lang="ts">`), Vite 8, TypeScript, Tailwind CSS, PrimeVue 4
- **State:** Pinia for auth store, Vue composables for feature-level state
- **Routing:** Vue Router with `createWebHashHistory()`
- **Desktop:** Tauri v2 with Rust backend

### Tauri IPC Pattern

All backend calls go through `src/tauri/commands.ts`:
- `safeInvoke<T>(command, args)` — wraps `tauri invoke`; throws if called outside the Tauri runtime (e.g., browser-only dev)
- `canUseTauriRuntime()` — guards UI branches that require Tauri
- `friendlyError(error)` — converts Tauri error strings to user-readable messages

Tauri commands are registered in `src-tauri/src/lib.rs` and implemented across three command modules:
- `commands/import_commands` — CSV import, batch listing, previewing monthly reports
- `commands/system_commands` — system info
- `commands/xlsx_markdown_commands` — Excel → Markdown conversion

### Frontend Structure

Feature-based folder structure:
```
src/
├─ app/
│  ├─ router/          # Vue Router setup (hash mode) + route definitions
│  ├─ stores/          # Pinia stores (auth)
│  └─ plugins/         # PrimeVue + Pinia plugin registration
├─ features/           # Feature modules (one per domain)
│  ├─ overview/        # Overview dashboard
│  ├─ projects/        # Project list + detail
│  ├─ skills/          # Skill catalog management
│  ├─ issues/          # Issue backlog + CSV import
│  ├─ import-csv/      # CSV data import
│  ├─ import-reports/  # Import report history + detail
│  ├─ xlsx-markdown/   # Excel → Markdown converter
│  ├─ daily-notes/     # Daily work notes calendar
│  ├─ daily-report/    # Daily hour input grid
│  ├─ settings/        # User settings + theme
│  └─ auth/            # Login page
├─ shared/
│  ├─ components/      # Sidebar, Header, BottomBar, StartupScreen, MessageBanner, etc.
│  ├─ composables/     # useAppShell (bootstrap, system info polling)
│  ├─ utils/           # timeMath.ts
│  ├─ config/          # appConfig.ts
│  └─ types/           # statistics.ts (all shared TS types)
├─ tauri/
│  ├─ commands.ts      # safeInvoke, canUseTauriRuntime, friendlyError
│  └─ events.ts        # (placeholder for Tauri events)
├─ App.vue             # App shell: sidebar + header + router-view + bottom bar
├─ main.ts             # Vue entry point
└─ styles.css          # Tailwind + CSS variable theming + PrimeVue overrides
```

Each feature folder follows the pattern:
- `components/` — Vue SFC pages
- `composables/` — `use<Feature>()` composable owning state, data fetching, and Tauri calls

**Routing** — Vue Router with `createWebHashHistory()`. Routes defined in `src/app/router/routes.ts`. Only `/settings` requires authentication. Navigation guard in `src/app/router/index.ts`.

**Auth** — Pinia store (`src/app/stores/auth.ts`). Login stores a `{ username }` object in localStorage (`pjjyuji.auth.session`). Protected routes redirect to `/login` with a `returnPath`.

**Layout shell** (`src/App.vue`) — collapsible sidebar (240px expanded / 72px collapsed) + header + `<router-view>` + bottom bar.

### Rust Backend Structure

Clean layered architecture under `src-tauri/src/`:
```
commands/       ← Tauri #[tauri::command] handlers (thin, delegate to services)
services/       ← Business logic (import_csv, monthly_report, system, project, xlsx_markdown)
database/       ← Persistence layer (JSON file store, CSV reading)
infrastructure/ ← Low-level I/O utilities (encoding_rs Shift-JIS, file path resolution)
utils/          ← Network and time helpers
app/            ← AppError type and Result alias
```

Data flow: `commands → services → database → infrastructure`

### Styling

Tailwind CSS with CSS variable–based theming. Colors like `bg-canvas` and `text-ink` are custom CSS variables set in `src/styles.css` and referenced in `tailwind.config.js`. Use these semantic tokens rather than raw Tailwind palette colors for consistency with the app's theme.

PrimeVue 4 with Aura theme preset, dark mode via `[data-theme='dark']` selector.

### App Window

Default size: 1200×760, minimum: 980×640. Design features to fit within minimum dimensions.

## Key Files

| File | Role |
|------|------|
| `src/App.vue` | App shell with layout grid and auth routing |
| `src/app/router/routes.ts` | All route definitions |
| `src/tauri/commands.ts` | Tauri IPC wrapper — always use `safeInvoke` for backend calls |
| `src/shared/types/statistics.ts` | Core TypeScript types shared across the app |
| `src/app/stores/auth.ts` | Pinia auth store |
| `src-tauri/src/lib.rs` | Tauri command registration |
| `src-tauri/tauri.conf.json` | App config (name, window size, build commands) |
| `scripts/xlsx_spec_to_markdown.py` | Python helper for Excel → Markdown conversion |
