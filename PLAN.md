# Plan

## What's built

- **Feed** (`/`): Infinite-scroll daily workout log, drag-to-reorder exercises and sets, PR badges
- **Calendar** (`/calendar`): Month view with activity dots, navigates to feed date
- **Exercise tracker** (`/exercise/[id]/[date]`): Full CRUD for sets, PR tracking, history, graph (estimated 1RM), PRs table (nRM viewer)
- **Body tracker** (`/body/[date]`): Log measurements per metric, auto-derived body fat % (Navy formula) and FFMI, history view
- **Settings** (`/settings`): FitNotes CSV import wizard, delete all data
- **Light/dark mode**: CSS variables defined; `dark_mode` stored in DB but not yet wired to the UI

---

## Database schema (current)

```sql
categories          -- exercise categories (Abs, Back, Biceps, ...)
exercises           -- exercise library (name, category_id)
workouts            -- one row per date
workout_exercises   -- which exercises appear on a date, with order
sets                -- individual sets (weight_kg, reps, was_pr_at_time, is_current_pr)

body_metrics        -- metric definitions (Weight kg, Waist cm, Body Fat %, ...)
body_measurements   -- logged values (date, measure_id, value)

user_settings       -- single row: height_cm, unit, dark_mode, estimate_body_fat, sex
```

**PR logic**: A set `(weight, reps)` is a PR if no other set for that exercise has `weight >= x AND reps >= n` — the Pareto frontier of the (weight, reps) space.
- `was_pr_at_time`: was this a PR when logged? Never cleared.
- `is_current_pr`: is this still a PR today? Recomputed for all sets of an exercise on every insert/edit/delete.

---

## Remaining features

### 0. Migration infrastructure ✓

Implemented in `src-tauri/src/database.rs`. `SCHEMA_VERSION` is a `const u32` at the top of the file. `run_migrations` reads `PRAGMA user_version`, returns an error if the DB is ahead of the code (downgrade guard), and runs each numbered `migrate_N` function under the condition `current < N && N <= SCHEMA_VERSION`. Version is bumped immediately after each migration succeeds. Current version: **1** (full initial schema).

Pending migrations to add as features land:
```
v2: ALTER TABLE user_settings ADD COLUMN season_start TEXT DEFAULT '01-01'
v3: ALTER TABLE user_settings ADD COLUMN use_seasons BOOLEAN DEFAULT true
v4: ALTER TABLE sets ADD COLUMN is_season_pr BOOLEAN DEFAULT false
v5: CREATE TABLE templates / template_exercises
```

---

### 1. Create and manage exercises

Currently exercises can only enter the DB via the FitNotes import. There is no UI for adding exercises manually. This is required before using the app without importing data.

**Commands** (all implemented):
- `create_exercise(name, category_id) → i64`
- `delete_exercise(id)` — blocked if sets exist for this exercise
- `rename_exercise(id, name)` — blocked if name already taken (UNIQUE constraint); returns "Exercise not found" if id invalid
- `create_category(name) → i64`
- `delete_category(id)` — blocked if exercises or sets exist in the category
- `rename_category(id, name)` — blocked if name already taken; returns "Category not found" if id invalid
- `merge_exercise_into_existing(from_id, to_name)` — re-points all sets and workout_exercises from `from` to `to`, handles the unique constraint on `(workout_id, exercise_id)` by deleting the `from` row where `to` already appears on the same day, deletes `from`, then calls `recompute_pr_flags(to_id)`

**Rename conflict behavior**: both `exercises.name` and `categories.name` have UNIQUE constraints in the DB. A rename to a duplicate name returns a SQLite UNIQUE constraint error as an `Err(String)`. The frontend catches this and offers the user Merge / Cancel (exercises only; categories cannot be merged).

**Frontend**:
- Replace `AddExerciseModal.svelte` with a new route `src/routes/exercises/[date=date]/+page.svelte`
- The "+Add exercise" button on the feed navigates to `/exercises/[date]` instead of opening the modal; delete `AddExerciseModal.svelte`
- The page has the same category → exercise drill-down as the old modal
- Header of each view has a "+ New" button (create category / create exercise in current category)
- Each list item has a ⋯ menu with Rename and Delete actions
- Rename: inline text input replacing the item; on UNIQUE error show "An exercise named X already exists — merge Y into X?" with Merge / Cancel
- Delete: confirmation prompt; shows a blocking error if the item has sets (exercise) or exercises/sets (category)
- Tapping an exercise item (outside the ⋯ menu) navigates to `/exercise/[id]/[date]` using `replaceState` so the browser history goes feed → exercise, skipping the exercises browser
- The exercise page's back button already goes to `/?date=[date]` (the feed) — no change needed there

---

### 2. Body measurements import

FitNotes exports body tracker data as CSV (exact column format TBD — inspect a real export before implementing; likely `Date, Measurement, Value, Unit`). Import follows the same multi-step wizard pattern as the exercise import.

**Commands**:
- `parse_body_measurements_csv(csv_text) → { rows: Vec<ParsedBodyRow>, unknown_metrics: Vec<String> }`
- `import_body_measurement_rows(rows: Vec<ResolvedBodyRow>) → { imported, skipped }`
  - Uses `INSERT OR IGNORE` to skip duplicate (date, metric_id) pairs

**Frontend** — same phase-based state machine as the exercise import in `src/routes/settings/+page.svelte`:
1. File picker → parse
2. Resolve unknowns: for each unrecognized metric name, user maps it to an existing body metric or skips it
3. Preview count → confirm → import → result

---

### 3. Android build

Tauri 2 has first-class Android support. The app logic (Svelte frontend + Rust backend) is unchanged — Tauri handles the platform layer. This is the first time building for mobile, so setup is the main effort.

**One-time setup** (developer machine):
1. Install Android Studio (includes the Android SDK and emulator)
2. Install Android NDK via the SDK Manager in Android Studio (Tauri needs this to cross-compile Rust)
3. Set environment variables: `ANDROID_HOME` (SDK path) and `NDK_HOME` (NDK path)
4. Install Rust Android targets: `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`
5. Run `pnpm tauri android init` in the project root — this generates an `src-tauri/gen/android/` directory with the Android project

**Development**:
- `pnpm tauri android dev` — builds and deploys to a connected device or emulator with hot reload
- The SQLite DB file path (currently resolved via `app.path().app_data_dir()`) works on Android — Tauri maps it to the app's internal storage

**Things to verify/fix before Android**:
- Touch targets: the current `-`/`+` buttons and row heights may be too small for fingers — aim for at least 44px tap targets
- The `svelte-dnd-action` drag-and-drop library for reordering sets/exercises uses mouse events; verify it works on touch or find a touch-compatible alternative
- File picker for CSV import uses a native dialog (`tauri-plugin-dialog`) — verify the Android plugin is included in `Cargo.toml` and `tauri.conf.json` capabilities
- The app header/nav assumes a desktop window width; test on phone viewport (360–390px wide)

**Build for distribution**:
- `pnpm tauri android build` — produces an APK or AAB
- For personal use, install the APK directly (no Play Store needed): enable "Install from unknown sources" on the device, transfer the APK, install

**Files**:
- `src-tauri/tauri.conf.json` — may need `android` section for permissions (storage, etc.)
- `src-tauri/Cargo.toml` — verify mobile-compatible plugin versions
- CSS — audit touch target sizes

**Known issues to fix**:

- *Transitions feel sudden*: No press feedback or page animations on mobile. Two parts:
  1. Press feedback — add `:active` CSS states to buttons and list items (e.g. `transform: scale(0.97)` with a short `transition`). Remove the default `-webkit-tap-highlight-color` globally (already done).
  2. Page transitions — use the View Transitions API via SvelteKit's `onNavigate` hook in `+layout.svelte`. Wrap `document.startViewTransition()` around the navigation completion; define `::view-transition-old/new(root)` animations in `app.css` for a subtle slide-fade (~150ms). The API is supported in the Chromium WebView version Tauri uses on modern Android; guard with `if (!document.startViewTransition) return` for safety.

- *DnD triggers on full card body instead of handle only*: `svelte-dnd-action` initiates drag on any `pointerdown` within the zone. Fix: set `dragDisabled={isDragDisabled}` on the dndzone (default `true`), then set it `false` only on `pointerdown` of the `≡` handle element and back to `true` in the `onfinalize` handler. This restricts drag initiation to the handle. The drag handle (`≡`) is already rendered in `DayCard.svelte`.


**iOS** (lower priority): Similar process — requires a Mac with Xcode, an Apple Developer account for device testing, and `pnpm tauri ios init`. Defer until Android is working.

---

### 4. Settings menu

The `/settings` page currently only has import and delete-all. Expose the user profile stored in `user_settings`.

**Commands**:
- `get_settings_frontend() → SettingsResponse` — needs a new serializable struct (current `Settings` struct is not `Serialize`)
- `update_settings(height_cm, unit, dark_mode, estimate_body_fat, sex, season_start, use_seasons)`

**Frontend** — new "Profile" section in `src/routes/settings/+page.svelte`:
- Height (number input)
- Sex (male/female)
- Weight unit (kg/lbs)
- Estimate body fat (toggle)
- Dark mode (toggle) — immediately applies `document.documentElement.dataset.theme`
- Season start + use seasons toggle (once feature 5 lands)

Load on mount; save on change (debounced or on blur). Apply dark mode in `src/routes/+layout.svelte` on boot.

---

### 5. Complete body tracker

Graph and PRs tabs exist in `BodyHeader.svelte` but have no routes yet.

**Graph** (`/body/graph`):
- New command: `get_body_metric_graph_data(metric_id, from_date, to_date) → Vec<DatedValue>`
- New route `src/routes/body/graph/` — reuse the graph pattern from `exercise/[id]/graph`, with a metric switcher dropdown

**PRs** (`/body/prs`):
- "All-time best value per metric"
- New command: `get_body_prs() → Vec<{metric, value, date}>`
- New route `src/routes/body/prs/` — simple table, one row per metric

**Files**: `src-tauri/src/commands/body.rs`, `src-tauri/src/lib.rs`, new `src/routes/body/graph/` and `src/routes/body/prs/`, `src/lib/body.ts`

---

### 6. Workout templates

Save a named list of exercises as a reusable template; apply it to any date to pre-populate the day's workout.

**DB changes** (migration 5):
```sql
CREATE TABLE templates (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE template_exercises (
    id             INTEGER PRIMARY KEY,
    template_id    INTEGER NOT NULL REFERENCES templates(id),
    exercise_id    INTEGER NOT NULL REFERENCES exercises(id),
    exercise_order INTEGER NOT NULL,
    UNIQUE(template_id, exercise_id)
);
```

**Commands**:
- `create_template(name) → i64`
- `save_day_as_template(date, name) → i64` — snapshot current workout_exercises for a date
- `list_templates() → Vec<{id, name, exercises[]}>`
- `delete_template(id)`
- `apply_template(template_id, date)` — adds all template exercises to the date, then inserts one pre-filled set per exercise using the last logged weight/reps for that exercise (fallback: 0 kg, 5 reps if no history)

**Frontend**:
- New route `src/routes/templates/` — list, delete, create from current day
- New `src/lib/TemplateModal.svelte` — picker modal, similar to `AddExerciseModal`
- `src/lib/DayCard.svelte` — "Use template" button alongside "Add exercise"

---

### 7. Season-wise personal bests

A season is a 1-year window starting from a user-configured month/day (MM-DD), recurring annually. The current season = from the most recent occurrence of that date to today.

**DB changes** (migrations 2–4):
```sql
ALTER TABLE user_settings ADD COLUMN season_start TEXT DEFAULT '01-01';
ALTER TABLE user_settings ADD COLUMN use_seasons  BOOLEAN DEFAULT true;
ALTER TABLE sets           ADD COLUMN is_season_pr BOOLEAN DEFAULT false;
```

**Logic**: `is_season_pr` uses the same Pareto-frontier algorithm as `is_current_pr`, but scoped only to sets within the current season window. Recomputed on every set insert/edit/delete via a new `recompute_season_prs(conn, exercise_id)` helper.

**Commands**:
- `get_season_rep_maxes(exercise_id) → Vec<RepMax>`
- Update `upsert_set` / `delete_set` to also recompute season PRs

**Frontend**:
- PRs page (`/exercise/[id]/prs`): add a second column "This season" alongside "All time" for each rep count row. If `use_seasons` is false (settings toggle), hide the season column.
- Settings page: season start MM-DD input + use_seasons toggle
- No "SPR" badge on set rows — keep the feed clean

---

### 8. Analysis page

New route `/analysis`. Three sections:

**A. Correlation explorer** — pick two series (any body metric or exercise estimated 1RM), plot on a dual-axis or scatter chart. New command: `get_aligned_series(series_a, series_b, from, to) → Vec<{date, a, b}>` using forward-fill for dates where one series has no data.

**B. Summary statistics** — per body metric: current value, 30/90-day delta, all-time min/max. New command: `get_body_summary_stats() → Vec<{metric, current, delta_30d, delta_90d, min, max}>`.

**C. Trend lines** — linear regression overlay on any time-series chart, computed client-side (least-squares on already-fetched data, no new backend command needed).

Add an analysis icon to the header nav in `src/routes/+layout.svelte`.

---

### 9. Data safety

Three layers of protection, all local, no server required.

**WAL mode** (one-liner, do immediately):

SQLite's default write mode has a crash window where the DB file can be left corrupted if the app is killed mid-write. WAL (Write-Ahead Log) eliminates this by writing new data to a separate log file first, leaving the main DB file untouched until the write is fully committed. Enable once at DB initialization:

```sql
PRAGMA journal_mode=WAL;
```

Side effect: SQLite creates two companion files alongside the DB (`-wal` and `-shm`). These must travel with the `.db` file — backup and export logic must either copy all three, or run `PRAGMA wal_checkpoint(TRUNCATE)` first to flush the log into the main file before copying.

**Automatic local backups**:

On every app startup, before doing anything else, copy the DB to a timestamped backup file and prune old ones. Implemented entirely in Rust at startup in `initialize_db()`:

```
app_data_dir/fitness-notes.db          ← live DB
app_data_dir/backups/2026-04-07.db     ← today's backup
app_data_dir/backups/2026-04-06.db
...                                     ← keep last 14
```

- Use `std::fs::copy` to copy the DB file (run `wal_checkpoint` first)
- Name backups by date (`YYYY-MM-DD.db`) so the same day never produces duplicates
- After copying, read the backup directory, sort by filename, delete any beyond the 14 most recent

This protects against a bad migration destroying data — the backup from before the migration is always there.

**Manual export and restore via settings**:

In `src/routes/settings/+page.svelte`, add a "Data" section with three actions:

- **Export backup**: opens a native save-file dialog (`tauri-plugin-dialog`), runs `wal_checkpoint`, copies the `.db` file to the chosen location. User can save to iCloud Drive, Google Drive, a USB drive, etc.
- **Restore from backup**: opens a native open-file dialog, user picks a `.db` file, app shuts down the current DB connection, replaces the live DB with the chosen file, reinitializes. Show a confirmation warning before doing this ("This will replace all current data").
- **View automatic backups**: list the backups in the backups directory with their dates, allow the user to restore any of them (same replace flow as above).

New Rust commands:
- `export_backup(dest_path: String)` — checkpoint + copy DB to dest
- `list_local_backups() → Vec<{filename, date}>` — reads backups directory
- `restore_backup(src_path: String)` — closes DB, replaces file, reopens; returns error if file is not a valid SQLite DB

---

## Implementation order

1. ~~**Migration infrastructure**~~ ✓ done
2. ~~**WAL mode + automatic backups**~~ ✓ done
3. **Create and manage exercises** — needed to use the app without a FitNotes import
4. **Body measurements import** — needed to bring in historical data before going mobile
5. **Android build** — get the app on device; do UI/touch fixes as needed
6. **Settings menu** — user profile, dark mode, manual export/restore
7. **Complete body tracker** (graph + PRs)
8. **Workout templates**
9. **Season PRs** — depends on settings
10. **Analysis page** — most complex, last
