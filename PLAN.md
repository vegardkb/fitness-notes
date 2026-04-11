# Plan

## What's built

- **Feed** (`/`): Infinite-scroll daily workout log, drag-to-reorder exercises and sets, PR badges
- **Calendar** (`/calendar`): Month view with activity dots, navigates to feed date
- **Exercise tracker** (`/exercise/[id]/[date]`): Full CRUD for sets, PR tracking, history, graph (estimated 1RM), PRs table (nRM viewer)
- **Body tracker** (`/body/[date]`): Log measurements per metric; derived metrics (BMI, Body Fat (Navy), FFMI (Navy)) computed on the fly from stored inputs, never written to DB; history view. `is_derived` flag in `body_metrics` controls read-only rendering. Derived metric dates reflect the most recently updated input.
- **Settings** (`/settings`): FitNotes exercise CSV import wizard, FitNotes Body Tracker CSV import wizard (resolve unknowns: create/map/skip), delete all data
- **Light/dark mode**: CSS variables defined; `dark_mode` stored in DB but not yet wired to the UI
- **Exercise/category management** (`/exercises/[date]`): Category → exercise drill-down, full CRUD (create, rename, delete, merge) for both categories and exercises via inline inputs and ⋯ context menus. Merge moves all sets/history to the target and recomputes PRs. Errors surface as toasts.
- **Migration infrastructure**: `run_migrations()` in `database.rs` with downgrade guard and per-version functions. WAL mode enabled at startup. Automatic daily backups (last 14 kept) on every launch.
- **Android build & deploy**: App runs on Android via Tauri. Touch targets, DnD, viewport, and file picker verified. Daily iteration via `pnpm tauri android dev` (USB, HMR). Release builds signed via keystore env vars; `deploy.sh` builds and installs in one step.

---

## Database schema (current)

```sql
categories (
    id      integer primary key,
    name    text not null unique
)

exercises (
    id          integer primary key,
    name        text not null unique,
    category_id integer not null references categories(id)
)

workouts (
    id   integer primary key,
    date text not null unique
)

workout_exercises (
    id             integer primary key,
    workout_id     integer not null references workouts(id),
    exercise_id    integer not null references exercises(id),
    exercise_order integer not null,
    unique (workout_id, exercise_id)
)

sets (
    id              integer primary key,
    workout_id      integer not null references workouts(id),
    exercise_id     integer not null references exercises(id),
    set_order       integer not null,
    weight_kg       real not null,
    reps            integer not null,
    notes           text,
    was_pr_at_time  boolean not null,
    is_current_pr   boolean not null
)

body_metrics (
    id         integer primary key,
    name       text not null unique,
    unit       text not null,
    is_derived boolean not null default false  -- derived metrics are never stored in body_measurements
)

body_measurements (
    id         integer primary key,
    date       text not null,
    value      real not null,
    measure_id integer not null references body_metrics(id)
)

user_settings (
    id                integer primary key check (id = 1),
    height_cm         integer not null default 178,
    unit              text not null default 'kg',         -- 'kg' | 'lbs'
    estimate_body_fat boolean not null default true,
    dark_mode         boolean not null default true,
    sex               text not null default 'male'        -- 'male' | 'female'
)
```

**PR logic**: A set `(weight, reps)` is a PR if no other set for that exercise has `weight >= x AND reps >= n` — the Pareto frontier of the (weight, reps) space.
- `was_pr_at_time`: was this a PR when logged? Never cleared.
- `is_current_pr`: is this still a PR today? Recomputed for all sets of an exercise on every insert/edit/delete.

---

## Remaining features

Pending migrations to add as features land:
```
v4: Schema refactor — recreate workouts, workout_exercises, sets (see section 5)
v5: ALTER TABLE user_settings ADD COLUMN season_start TEXT DEFAULT '01-01'
v6: ALTER TABLE user_settings ADD COLUMN use_seasons BOOLEAN DEFAULT true
v7: ALTER TABLE sets ADD COLUMN is_season_pr BOOLEAN DEFAULT false
v8: CREATE TABLE templates / template_exercises
```

---

### 5. Workout/exercise model refactor

**Problem**: Two constraints limit what the app can represent:
- `workouts.date` is `UNIQUE` — only one workout per day
- `workout_exercises` has `UNIQUE(workout_id, exercise_id)` — an exercise can only appear once per workout, making A→B→A structure impossible
- `sets` references `(workout_id, exercise_id)` as a composite rather than a specific exercise instance — there is no first-class "exercise instance with no sets", which makes templates awkward (applying a template would require dummy sets)

**Scope**: Remove both unique constraints, migrate `sets` to reference a `workout_exercise_id` FK, and keep `exercise_id` on `sets` as a denormalized column for fast PR queries. Multiple workouts per day is intentionally deferred — the schema will support it, but the frontend will not expose it yet (see end of document).

#### Schema changes (migration v4)

Three tables must be recreated — SQLite cannot drop constraints in-place:

```sql
-- workouts: add day_order, drop unique on date
CREATE TABLE workouts_new (
    id        integer primary key,
    date      text not null,
    day_order integer not null default 1
);
INSERT INTO workouts_new SELECT id, date, 1 FROM workouts;
DROP TABLE workouts; ALTER TABLE workouts_new RENAME TO workouts;

-- workout_exercises: drop unique(workout_id, exercise_id)
CREATE TABLE workout_exercises_new (
    id             integer primary key,
    workout_id     integer not null references workouts(id),
    exercise_id    integer not null references exercises(id),
    exercise_order integer not null
);
INSERT INTO workout_exercises_new SELECT * FROM workout_exercises;
DROP TABLE workout_exercises; ALTER TABLE workout_exercises_new RENAME TO workout_exercises;

-- sets: replace (workout_id, exercise_id) with workout_exercise_id
-- exercise_id kept denormalized for fast PR queries (WHERE exercise_id = ?)
CREATE TABLE sets_new (
    id                  integer primary key,
    workout_exercise_id integer not null references workout_exercises(id),
    exercise_id         integer not null references exercises(id),
    set_order           integer not null,
    weight_kg           real not null,
    reps                integer not null,
    notes               text,
    was_pr_at_time      boolean not null,
    is_current_pr       boolean not null
);
INSERT INTO sets_new
    SELECT s.id, we.id, s.exercise_id, s.set_order,
           s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
    FROM sets s
    JOIN workout_exercises we
      ON we.workout_id = s.workout_id AND we.exercise_id = s.exercise_id;
DROP TABLE sets; ALTER TABLE sets_new RENAME TO sets;
```

#### Backend changes (`src-tauri/src/`)

New command:
- `add_exercise_to_workout(workout_id, exercise_id) → workout_exercise_id` — inserts a `workout_exercises` row and returns its id; the frontend calls this before inserting the first set for a new exercise instance

Changed commands in `sets.rs`:
- `upsert_set` — takes `workout_exercise_id` instead of `(date, exercise_id)`; set_order MAX query scoped to `workout_exercise_id`
- `delete_set` — cleanup uses `workout_exercise_id` to decide whether to remove the `workout_exercises` row
- `reorder_sets` — scope changes from `(workout_id, exercise_id)` to `workout_exercise_id`

Changed commands in `workouts.rs`:
- `get_workout_for_date` — return type adds `workout_exercise_id` to each `ExerciseWithSets`
- `get_workouts_for_range` — same join change
- `get_active_dates` — add explicit `DISTINCT` on date (previously implicit from unique constraint)
- `reorder_exercises` — takes `workout_id` instead of `date` (date is no longer a unique workout key)

Changed commands in `exercises.rs`:
- `get_exercise_history`, `get_exercise_graph_data`, `get_rep_maxes`, `get_last_set` — join structure changes; queries using denormalized `sets.exercise_id` are unchanged
- `merge_exercise_into_existing` — `UPDATE sets SET exercise_id = ?` plus `UPDATE workout_exercises SET exercise_id = ?`
- `delete_exercise`, `delete_category` — validation COUNT queries unchanged (use denorm `sets.exercise_id`)

Changed command in `import.rs`:
- `import_fitnotes_rows` — workout lookup changes from `INSERT OR IGNORE` (relied on unique date constraint) to `SELECT id FROM workouts WHERE date = ? AND day_order = 1`; set insertion uses `workout_exercise_id`

#### Frontend changes (`src/`)

**Route rename**: `src/routes/exercise/[id]/[date]/` → `src/routes/exercise/[workout_exercise_id]/`
The `(exercise_id, date)` pair is no longer a unique identifier once exercises can repeat within a day. `workout_exercise_id` uniquely identifies an exercise instance. Human-readable context (date, name) is loaded from `get_workout_for_date`.

**`src/lib/exercise.ts`**:
- `ExerciseWithSets` gains `workout_exercise_id: number`
- `Set` drops `workout_id`

**`src/lib/DayCard.svelte`**:
- Navigation: `goto(/exercise/${ex.exercise_id}/${date})` → `goto(/exercise/${ex.workout_exercise_id})`
- Adding an exercise: call `add_exercise_to_workout` to get `workout_exercise_id` before navigating
- `reorder_exercises` passes `workout_id` instead of `date`

**`src/routes/exercise/[workout_exercise_id]/+page.svelte`**:
- `upsert_set` and `reorder_sets` pass `workout_exercise_id`
- Exercise context (name, date) resolved from `workout_exercise_id` via the workout data

---

### 6. Settings menu

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

### 7. Complete body tracker

Graph and PRs tabs exist in `BodyHeader.svelte` but have no routes yet.

**Navy BF% and FFMI are derived-only** — never stored in `body_measurements`. They are computed on the fly from the logged weight, waist, neck (and hip for females) measurements whenever `estimate_body_fat` is enabled in settings. Users cannot manually edit these values. If a user wants to track body fat from another method (calipers, DEXA), they create a custom metric (e.g. "Body Fat (Calipers)") and log it as a regular measurement.

**Graph** (`/body/graph`):
- New command: `get_body_metric_graph_data(metric_id, from_date, to_date) → Vec<DatedValue>`
- New route `src/routes/body/graph/` — reuse the graph pattern from `exercise/[id]/graph`, with a metric switcher dropdown
- Derived metrics (Navy BF%, FFMI) appear in the switcher but their data is computed, not fetched from `body_measurements`

**PRs** (`/body/prs`):
- "All-time best value per metric" — excludes derived metrics (Navy BF%, FFMI have no stored rows to query)
- New command: `get_body_prs() → Vec<{metric, value, date}>`
- New route `src/routes/body/prs/` — simple table, one row per metric

**Files**: `src-tauri/src/commands/body.rs`, `src-tauri/src/lib.rs`, new `src/routes/body/graph/` and `src/routes/body/prs/`, `src/lib/body.ts`

---

### 8. Workout templates

Save a named list of exercises as a reusable template; apply it to any date to pre-populate the day's workout.

**DB changes** (migration v8):
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

### 9. Season-wise personal bests

A season is a 1-year window starting from a user-configured month/day (MM-DD), recurring annually. The current season = from the most recent occurrence of that date to today.

**DB changes** (migrations v5–v7):
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

### 10. Analysis page

New route `/analysis`. Three sections:

**A. Correlation explorer** — pick two series (any body metric or exercise estimated 1RM), plot on a dual-axis or scatter chart. New command: `get_aligned_series(series_a, series_b, from, to) → Vec<{date, a, b}>` using forward-fill for dates where one series has no data.

**B. Summary statistics** — per body metric: current value, 30/90-day delta, all-time min/max. New command: `get_body_summary_stats() → Vec<{metric, current, delta_30d, delta_90d, min, max}>`.

**C. Trend lines** — linear regression overlay on any time-series chart, computed client-side (least-squares on already-fetched data, no new backend command needed).

Add an analysis icon to the header nav in `src/routes/+layout.svelte`.

---

### 11. Data safety

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
3. ~~**Create and manage exercises**~~ ✓ done
4. ~~**Android build**~~ ✓ done
5. ~~**Body metrics overhaul**~~ ✓ done — is_derived, FitNotes renames, on-the-fly derived metrics, write guard in upsert
6. ~~**Body measurements import**~~ ✓ done
7. ~~**Android build/test workflow**~~ ✓ done — `tauri android dev` for iteration, `deploy.sh` for release
8. **Workout/exercise model refactor** — workout_exercise_id, exercise repetition, template foundation
9. **Settings menu** — user profile, dark mode, manual export/restore
10. **Complete body tracker** (graph + PRs)
11. **Workout templates** — depends on refactor (section 8)
12. **Season PRs** — depends on settings
13. **Analysis page** — most complex, last

---

## Deferred: multiple workouts per day

The schema refactor (section 5) removes the `UNIQUE` constraint on `workouts.date` and adds a `day_order` column, so the database already supports multiple workouts per day. This is intentionally not exposed in the UI yet.

When/if the time comes, the remaining work is frontend-only:

- **Feed** (`src/lib/DayCard.svelte`): group exercises under named workout blocks ("Workout 1", "Workout 2"); add an "Add workout" button per day that calls a new `create_workout(date) → workout_id` command
- **`create_workout` command** (`src-tauri/src/commands/workouts.rs`): `INSERT INTO workouts (date, day_order) VALUES (?1, (SELECT COALESCE(MAX(day_order), 0) + 1 FROM workouts WHERE date = ?1))`
- **`reorder_exercises`**: already takes `workout_id` after the refactor, so multi-workout reordering works without further backend changes
- No migration needed — the schema is already correct after v4
