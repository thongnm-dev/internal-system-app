# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**PJ Yuji Statistics** — a Tauri 2.0 desktop application for internal project management and statistics tracking. The app imports CSV data, generates reports, tracks daily work, and converts Excel specs to Markdown.

## Commands

```bash
# Full development (Tauri + React hot reload)
npm run tauri:dev

# Frontend-only dev server (http://127.0.0.1:1420)
npm run dev

# Production desktop build
npm run tauri:build

# Frontend build only
npm run build
```

There are no test commands — the project has no test suite currently.

When developing the Rust backend, run Cargo commands from `src-tauri/`:
```bash
cd src-tauri && cargo check
cd src-tauri && cargo build
```

## Architecture

### Tauri IPC Pattern

All backend calls go through `src/core/tauriRuntime.ts`:
- `safeInvoke<T>(command, args)` — wraps `tauri invoke`; throws if called outside the Tauri runtime (e.g., browser-only dev)
- `canUseTauriRuntime()` — guards UI branches that require Tauri
- `friendlyError(error)` — converts Tauri error strings to user-readable messages

Tauri commands are registered in `src-tauri/src/lib.rs` and implemented across three command modules:
- `commands/import_commands` — CSV import, batch listing, previewing monthly reports
- `commands/system_commands` — system info
- `commands/xlsx_markdown_commands` — Excel → Markdown conversion

### Frontend Structure

**Controller hooks** (`src/controller/`) — each page has a dedicated `use*Controller` hook that owns data fetching, state, and Tauri calls. Pages are thin — they only render what the controller provides.

**Routing** — custom hash-based router via `useHashRouter`. Routes are defined in `src/router/routes.ts`. Only `/settings` requires authentication (`requiresAuth: true`). `routeByPath()` handles sub-paths like `/projects/:id` and `/import-reports/:id`.

**Auth** — simple React Context store (`src/stores/authStore.tsx`). Login stores a `{ username }` object; no real token/session. Protected routes redirect to `/login` with a `returnPath` to restore navigation after login.

**Layout shell** (`src/App.tsx`) — collapsible sidebar (240px expanded / 72px collapsed) + header + bottom bar. `MainDetailArea` switches page content based on `activeMenu`.

### Rust Backend Structure

Clean layered architecture under `src-tauri/src/`:
```
commands/       ← Tauri #[tauri::command] handlers (thin, delegate to domain)
domain/         ← Business logic (monthly_report, import_csv, system)
infrastructure/ ← File I/O and CSV parsing with encoding_rs (handles Shift-JIS)
utils/          ← Network and time helpers
app/            ← AppError type and Result alias
```

### Styling

Tailwind CSS with CSS variable–based theming. Colors like `bg-canvas` and `text-ink` are custom CSS variables set in `src/styles.css` and referenced in `tailwind.config.js`. Use these semantic tokens rather than raw Tailwind palette colors for consistency with the app's theme.

### App Window

Default size: 1200×760, minimum: 980×640. Design features to fit within minimum dimensions.

## Key Files

| File | Role |
|------|------|
| `src/App.tsx` | App shell with layout grid and auth routing |
| `src/router/routes.ts` | All route definitions |
| `src/core/tauriRuntime.ts` | Tauri IPC wrapper — always use `safeInvoke` for backend calls |
| `src/types/statistics.ts` | Core TypeScript types shared across the app |
| `src-tauri/src/lib.rs` | Tauri command registration |
| `src-tauri/tauri.conf.json` | App config (name, window size, build commands) |
| `scripts/xlsx_spec_to_markdown.py` | Python helper for Excel → Markdown conversion |
