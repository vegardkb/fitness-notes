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

SvelteKit file-based routes (all statically pre-rendered):
- `/` — Calendar view
- `/day/[date]` — Day view (YYYY-MM-DD)
- `/exercise/[id]` — Exercise view
- `/exercise/[id]/history` — Exercise history
- `/exercise/[id]/graph` — Exercise graph
- `/body` — Body tracker log
- `/body/graph` — Body tracker graphs
- `/import`, `/export` — Data tools

Selected date and other ephemeral UI state live in a Svelte store. Unit preference (kg/lb) and user height are stored in the `user_settings` DB table (single row), not in localStorage. Weight values are always stored in kg in the DB; the frontend converts to the display unit.

## PR Logic

A set `(weight=x, reps=n)` is a PR if no other set for that exercise has `weight >= x AND reps >= n` — the Pareto frontier of the (weight, reps) space. Two flags are stored per set:

- `was_pr_at_time`: was this a PR when logged? Never cleared after being set.
- `is_current_pr`: is this still a PR today? Recomputed for all sets of an exercise on every insert/edit/delete.

See `PLAN.md` for the full algorithms.
