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
