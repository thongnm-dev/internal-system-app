# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Management Systems** — a Tauri 2.0 desktop application (v1.0.9) for internal project management, daily reporting, cloud storage, AI usage tracking, and developer tools. Built for a team workflow with PostgreSQL backend, S3 integration, Backlog API integration, and role-based access control.

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

# Bump app version (package.json + tauri.conf.json)
npm run version:bump
```

There are no test commands — the project has no test suite currently.

When developing the Rust backend, run Cargo commands from `src-tauri/`:
```bash
cd src-tauri && cargo check
cd src-tauri && cargo build
```

## Architecture

### Tech Stack

- **Frontend:** Vue 3 (Composition API with `<script setup lang="ts">`), Vite 8, TypeScript, Tailwind CSS 3, PrimeVue 4
- **State:** Pinia stores (auth, menu, members), Vue composables for feature-level state
- **Routing:** Vue Router with `createWebHashHistory()`
- **Desktop:** Tauri v2 with Rust backend
- **Database:** PostgreSQL (accessed exclusively through stored procedures)
- **Tauri Plugins:** dialog, updater, process, notification

### Tauri IPC Pattern

All backend calls go through modular command files under `src/tauri/commands/`:
- `_base.ts` — core utilities: `safeInvoke<T>()`, `canUseTauriRuntime()`, `friendlyError()`
- One file per domain (e.g., `auth.ts`, `project.ts`, `s3.ts`, `ai-usage.ts`)
- `index.ts` — re-exports all command modules + base utilities

`src/tauri/events.ts` — Tauri event listeners for backend-pushed events (`ai-usage-updated`, `s3-new-documents`).

Tauri commands are registered in `src-tauri/src/lib.rs` and implemented across 24 command modules under `src-tauri/src/commands/`.

### Frontend Structure

```
src/
├─ app/
│  ├─ router/              # Vue Router setup (hash mode) + route definitions
│  ├─ stores/              # Pinia stores (auth, menu, members)
│  └─ plugins/             # PrimeVue + Pinia plugin registration
├─ features/               # Feature modules (one folder per domain)
│  ├─ overview/            # Overview dashboard
│  ├─ projects/            # Project CRUD, tasks, import, reports
│  ├─ daily-notes/         # Daily work notes calendar
│  ├─ daily-report/        # Daily hour input grid
│  ├─ issues/              # Issue backlog + Backlog API integration
│  ├─ tools/               # Excel→MD, Copy Tools, SQL Editor, Explore Faster, Check Monthly Report
│  ├─ cloud/               # S3 browser, upload, download, upload/download history
│  ├─ ai-agent/            # AI Chat, AI Usage tracking
│  ├─ governance/          # Menus, Users, Roles, Permissions, Logs, AppConfig, StoreProcedure, Skills
│  ├─ settings/            # User settings + theme
│  └─ auth/                # Login, Forgot Password
├─ shared/
│  ├─ components/          # AppSidebar, AppHeader, AppBottomBar, StartupScreen, GlobalLoading,
│  │                       # AppToast, MessageBanner, SummaryCards, NetworkStatusBanner,
│  │                       # DatabaseConfigScreen, DatabaseErrorScreen, ConnectionErrorScreen
│  ├─ composables/         # useAppShell, useAppUpdater, useDatabaseStatus, useNetworkStatus,
│  │                       # useNavigationHistory, useToast, useGlobalLoading
│  ├─ utils/               # timeMath.ts
│  └─ config/              # appConfig.ts, themeTokens.ts
├─ _/
│  └─ types/               # All shared TypeScript types (one file per domain)
├─ tauri/
│  ├─ commands/            # Modular IPC wrappers (_base.ts + per-domain files)
│  └─ events.ts            # Backend event listeners (AI usage, S3 notifications)
├─ App.vue                 # App shell: sidebar + header + router-view + bottom bar
├─ main.ts                 # Vue entry point
└─ styles.css              # Tailwind + CSS variable theming + PrimeVue overrides
```

Each feature folder follows the pattern:
- `components/` — Vue SFC pages
- `composables/` — `use<Feature>()` composable owning state, data fetching, and Tauri calls
- `utils/` — (optional) feature-specific utilities (e.g., `tools/utils/` for SQL highlighting/formatting)

**Routing** — Vue Router with `createWebHashHistory()`. Routes defined in `src/app/router/routes.ts`. Governance and Settings routes require authentication. Navigation guard in `src/app/router/index.ts`.

**Auth** — Pinia store (`src/app/stores/auth.ts`). Login via backend auth commands against PostgreSQL. Forgot password flow with email-based reset codes. Protected routes redirect to `/login` with a `returnPath`.

**Layout shell** (`src/App.vue`) — collapsible sidebar (240px expanded / 72px collapsed) + header + `<router-view>` + bottom bar.

### Rust Backend Structure

Layered architecture under `src-tauri/src/`:
```
commands/       ← Tauri #[tauri::command] handlers (thin, delegate to services)
services/       ← Business logic (one service per domain)
database/       ← PostgreSQL store layer (calls stored procedures)
utils/          ← api_client, csv_reader, time, app_config, pgsql_connect, network, email
app/            ← AppError type, Result alias, consts
```

Data flow: `commands → services → database → utils`

**Background services** (started in `lib.rs` setup):
- `ai_usage_service::run_poll_loop` — polls AI usage data and emits `ai-usage-updated` events
- `s3_watch_service::run_poll_loop` — watches S3 storage and emits `s3-new-documents` notifications

**Database initialization** — `database/startup_store.rs::init()` runs in debug mode to create tables and install stored procedures.

**IMPORTANT — Database access must go through stored procedures:**
- The `database/` layer (store files) must NEVER write raw SQL (SELECT/INSERT/UPDATE/DELETE) directly. All queries must call PostgreSQL stored procedures (functions).
- Stored procedures are defined as `.sql` files in `docs/store-procedure/`, naming convention: `sp_{domain}_{action}.sql`.
- Every new SP must be registered in **both** `services/sp_management_service.rs` (`all_procedures()`) and `database/startup_store.rs` (`ensure_stored_procedures()`).
- The StoreProcedurePage.vue UI reads the list from `sp_management_service` — if a new SP is not registered there, it won't appear in the management screen.

### Styling

Tailwind CSS with CSS variable–based theming. Colors like `bg-canvas` and `text-ink` are custom CSS variables set in `src/styles.css`, mapped in `src/shared/config/themeTokens.ts`, and referenced in `tailwind.config.js`. Use these semantic tokens rather than raw Tailwind palette colors for consistency with the app's theme.

PrimeVue 4 with Aura theme preset, dark mode via `[data-theme='dark']` selector.

### App Window

Default size: 1200×760, minimum: 980×600. Opens maximized. Design features to fit within minimum dimensions.

## Key Files

| File | Role |
|------|------|
| `src/App.vue` | App shell with layout grid and auth routing |
| `src/app/router/routes.ts` | All route definitions + breadcrumb logic |
| `src/tauri/commands/_base.ts` | Tauri IPC core — `safeInvoke`, `canUseTauriRuntime`, `friendlyError` |
| `src/tauri/commands/index.ts` | Re-exports all IPC command modules |
| `src/tauri/events.ts` | Backend event listeners (AI usage, S3) |
| `src/_/types/` | All shared TypeScript types (one file per domain) |
| `src/app/stores/auth.ts` | Pinia auth store |
| `src/app/stores/menu.ts` | Pinia menu config store |
| `src/app/stores/members.ts` | Pinia members store |
| `src-tauri/src/lib.rs` | Tauri command registration + plugin setup + background services |
| `src-tauri/tauri.conf.json` | App config (name, window size, build commands, updater) |
| `docs/store-procedure/` | All PostgreSQL stored procedure definitions |
