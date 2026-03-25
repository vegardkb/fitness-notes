# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

Use `pnpm` as the package manager.

```bash
# Run the desktop app in development mode (starts Vite dev server + Tauri window)
pnpm tauri dev

# Type-check Svelte/TypeScript
pnpm check
pnpm check:watch

# Build the frontend only
pnpm build

# Build the full desktop app for distribution
pnpm tauri build
```

There are no test commands configured yet.

```bash
# Run Rust tests (from src-tauri/)
cargo test
```

## Architecture

This is a **Tauri 2 desktop app** combining a SvelteKit frontend with a Rust backend.

**Frontend** (`src/`): SvelteKit with Svelte 5 and TypeScript, configured for static site generation (`prerender = true`, `ssr = false` via `adapter-static`). No server-side rendering — the output is a static bundle loaded by Tauri.

**Backend** (`src-tauri/src/`): Rust code exposed to the frontend via `#[tauri::command]` functions. Frontend calls these using `invoke("command_name", { args })` from `@tauri-apps/api/core`.

**Communication pattern**:
```typescript
// Frontend (Svelte)
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("greet", { name: "world" });
```
```rust
// Backend (Rust) in src-tauri/src/lib.rs
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

New Tauri commands must be registered in the `generate_handler![]` macro in `lib.rs`.

**Dev setup**: Tauri starts the Vite dev server (`pnpm dev`) automatically on port 1420 before opening the app window. HMR works during `tauri dev`.

## Database

SQLite via `rusqlite` in the Rust backend. The frontend never issues SQL directly — all data access goes through typed Tauri commands that return serialized Rust structs. The DB file lives in the Tauri app data directory, resolved at runtime:

```rust
let db_path = app.path().app_data_dir()?.join("fitness-notes.db");
```

```toml
# src-tauri/Cargo.toml
rusqlite = { version = "0.31", features = ["bundled"] }
rusqlite_migration = "1"  # optional, for migration management
```

SQL migration files live in `src-tauri/migrations/` and are applied at startup. The `rusqlite` connection is held in a `Mutex<Connection>` managed by Tauri's state system (`app.manage(...)`) and accessed in commands via `State<Mutex<Connection>>`.

## Routing Structure

- `/` — Feed (infinite-scroll list of day cards, newest first). `prerender = false`.
- `/calendar` — Month calendar with workout activity dots. Navigates to `/?date=YYYY-MM-DD`.
- `/exercise/[id]/[date]` — Sets for one exercise on one date. Back button returns to `/`.

The day view no longer exists as a standalone route — it is `src/lib/DayCard.svelte`, rendered as a card inside the feed.

**Key components:**
- `src/lib/DayCard.svelte` — renders one day's exercises as a card. Fetches its own data on mount via `get_workout_for_date`. Accepts `date: string` prop.
- `src/lib/AddExerciseModal.svelte` — bottom-sheet modal for selecting category → exercise. Navigates to `/exercise/[id]/[date]` on selection.

**Feed scroll state** persists across client-side navigations using module-level variables in `+page.svelte` (`<script module>`). Back-navigation from the exercise view restores the previous scroll position and loaded date window. Calendar navigation (`?date=`) triggers a fresh load centered on the target date.

**`get_workouts_for_range(from_date, to_date)`** is the batch command for fetching workout data across a date range in one SQLite query. The feed uses individual `get_workout_for_date` calls per DayCard; `get_workouts_for_range` is available for features that need bulk data.

Unit preference (kg/lb) and user height are stored in the `user_settings` DB table (single row), not in localStorage. Weight values are always stored in kg in the DB; the frontend converts to the display unit.

## PR Logic

A set `(weight=x, reps=n)` is a PR if no other set for that exercise has `weight >= x AND reps >= n` — the Pareto frontier of the (weight, reps) space. Two flags are stored per set:

- `was_pr_at_time`: was this a PR when logged? Never cleared after being set.
- `is_current_pr`: is this still a PR today? Recomputed for all sets of an exercise on every insert/edit/delete.

See `PLAN.md` for the full algorithms.
