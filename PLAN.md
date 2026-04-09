# Plan

## What's built

- **Feed** (`/`): Infinite-scroll daily workout log, drag-to-reorder exercises and sets, PR badges
- **Calendar** (`/calendar`): Month view with activity dots, navigates to feed date
- **Exercise tracker** (`/exercise/[id]/[date]`): Full CRUD for sets, PR tracking, history, graph (estimated 1RM), PRs table (nRM viewer)
- **Body tracker** (`/body/[date]`): Log measurements per metric, auto-derived body fat % (Navy formula) and FFMI, history view
- **Settings** (`/settings`): FitNotes CSV import wizard, delete all data
- **Light/dark mode**: CSS variables defined; `dark_mode` stored in DB but not yet wired to the UI
- **Exercise/category management** (`/exercises/[date]`): Category → exercise drill-down, full CRUD (create, rename, delete, merge) for both categories and exercises via inline inputs and ⋯ context menus. Merge moves all sets/history to the target and recomputes PRs. Errors surface as toasts.
- **Migration infrastructure**: `run_migrations()` in `database.rs` with downgrade guard and per-version functions. WAL mode enabled at startup. Automatic daily backups (last 14 kept) on every launch.
- **Android build**: App runs on Android via Tauri. Touch targets, DnD, viewport, and file picker verified. Build via `pnpm tauri android build`; iOS deferred.

---

## Database schema (current)

```sql
categories          -- exercise categories (Abs, Back, Biceps, ...)
exercises           -- exercise library (name, category_id)
workouts            -- one row per date
workout_exercises   -- which exercises appear on a date, with order
sets                -- individual sets (weight_kg, reps, was_pr_at_time, is_current_pr)

body_metrics        -- metric definitions (name, unit, is_derived); derived metrics have no rows in body_measurements
body_measurements   -- logged values (date, metric_id, value); never contains rows for derived metrics

user_settings       -- single row: height_cm, unit, dark_mode, estimate_body_fat, sex
```

**PR logic**: A set `(weight, reps)` is a PR if no other set for that exercise has `weight >= x AND reps >= n` — the Pareto frontier of the (weight, reps) space.
- `was_pr_at_time`: was this a PR when logged? Never cleared.
- `is_current_pr`: is this still a PR today? Recomputed for all sets of an exercise on every insert/edit/delete.

---

## Remaining features

Pending migrations to add as features land:
```
v2: body_metrics overhaul — is_derived column, FitNotes renames, derived metric rows (see section 1)
v3: ALTER TABLE user_settings ADD COLUMN season_start TEXT DEFAULT '01-01'
v4: ALTER TABLE user_settings ADD COLUMN use_seasons BOOLEAN DEFAULT true
v5: ALTER TABLE sets ADD COLUMN is_season_pr BOOLEAN DEFAULT false
v6: CREATE TABLE templates / template_exercises
```

---

### 1. Body metrics overhaul

Two concerns addressed together in migration v2, since both must land before the import tool for correct metric name matching.

**a) Derived metrics**

Add `is_derived` to `body_metrics`. Derived metrics appear in `list_metrics` with `is_derived: true` so the UI renders them read-only (no edit/delete, no manual entry field). They are computed on the fly from other stored measurements — no rows are ever written to `body_measurements` for these `metric_id`s.

| Metric | Inputs | Formula |
|---|---|---|
| Body Fat (Navy) | waist, neck, height, sex (+ hip for female) | US Navy formula |
| FFMI (Navy) | weight, Body Fat (Navy), height | fat-free mass / height² |
| BMI | weight, height (from user_settings) | weight_kg / height_m² |

**b) FitNotes compatibility renames**

Rename bilateral metrics to left/right variants so FitNotes imports map without manual resolution. Existing measurements are preserved under the renamed metric.

| Before | After |
|---|---|
| Arm | Upper Arm (Left) |
| *(new)* | Upper Arm (Right) |
| *(new)* | Forearm (Left) |
| *(new)* | Forearm (Right) |
| Thigh | Thigh (Left) |
| *(new)* | Thigh (Right) |
| Calf | Calf (Left) |
| *(new)* | Calf (Right) |

**Migration v2 SQL**:
```sql
-- Add is_derived flag
ALTER TABLE body_metrics ADD COLUMN is_derived BOOLEAN NOT NULL DEFAULT false;

-- FitNotes compatibility renames (existing measurements follow the renamed row)
UPDATE body_metrics SET name = 'Upper Arm (Left)' WHERE name = 'Arm';
INSERT OR IGNORE INTO body_metrics (name, unit)
    SELECT 'Upper Arm (Right)', unit FROM body_metrics WHERE name = 'Upper Arm (Left)';

INSERT OR IGNORE INTO body_metrics (name, unit) VALUES ('Forearm (Left)', 'cm');
INSERT OR IGNORE INTO body_metrics (name, unit) VALUES ('Forearm (Right)', 'cm');

UPDATE body_metrics SET name = 'Thigh (Left)' WHERE name = 'Thigh';
INSERT OR IGNORE INTO body_metrics (name, unit)
    SELECT 'Thigh (Right)', unit FROM body_metrics WHERE name = 'Thigh (Left)';

UPDATE body_metrics SET name = 'Calf (Left)' WHERE name = 'Calf';
INSERT OR IGNORE INTO body_metrics (name, unit)
    SELECT 'Calf (Right)', unit FROM body_metrics WHERE name = 'Calf (Left)';

-- Derived metrics
INSERT OR IGNORE INTO body_metrics (name, unit, is_derived) VALUES ('Body Fat (Navy)', '%', true);
INSERT OR IGNORE INTO body_metrics (name, unit, is_derived) VALUES ('FFMI (Navy)', '', true);
INSERT OR IGNORE INTO body_metrics (name, unit, is_derived) VALUES ('BMI', 'kg/m²', true);
```

**Required changes to existing backend and frontend**:

*`src-tauri/src/commands/body.rs`*:
- **`list_metrics`**: add `is_derived` to the `SELECT` and return it. The `Metric` struct in `models.rs` needs a matching `is_derived: bool` field.
- **`upsert_body_measurement`**: remove the entire post-write block that calls `recompute_body_fat` and `recompute_ffmi` (lines 37–78). The command becomes a simple write with no derived side-effects.
- **`recompute_body_fat` / `recompute_ffmi`**: delete both functions entirely. The math functions (`body_fat_male`, `body_fat_female`, `calc_ffmi`) are kept — they move to serving on-the-fly reads.
- **`get_measurements_for_date` / `get_last_measurements_for_date`**: after fetching stored rows, compute and append derived metric values on the fly. For each derived metric whose inputs are present in the fetched set, compute the value and push a synthetic `Measurement` with `id: None` (no DB row). Guard on `settings.estimate_body_fat` for the Navy metrics; BMI only needs weight + height from settings.
- **`get_measurement_history`**: same appending pattern per date — after grouping each day's stored measurements, compute and append derived values. This means the history view will show Body Fat (Navy) / FFMI / BMI alongside manual entries without any stored rows.
- **`get_measure_id` / `get_measurement_id`**: delete both helpers (only used by the removed recompute functions).

*`src/lib/body.ts`*:
- Add `is_derived: boolean` to the `Metric` type.
- Add `is_derived` to the `Measurement` type (or derive it from `metric.is_derived`) so the frontend can check it without looking at `metric` every time.

*`src/routes/body/[date=date]/+page.svelte`*:
- In `fetchMeasurements`, `measurementsFilled` is built by merging `metrics` (all metrics) with `measurements` (stored values). Derived metrics will now appear in `measurements` already (computed by the backend), so the merge logic stays the same — but the row needs to be treated differently:
  - No `–` / `+` buttons
  - No editable `<input>`, just a display value
  - No save/delete toggle button
  - Visually distinct (e.g. muted or labelled "estimated")
- The `toggle`, `save`, and `deleteMeasurement` functions must guard against `row.metric.is_derived` and no-op if true.

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

### 4. Android build/test workflow

The current process — `pnpm tauri android build` → wait → sign manually → `adb install` → wait — has too much friction for iterative testing. Two things to fix: iteration speed and release signing.

**For iteration: use `tauri android dev` on a real device**

`pnpm tauri android dev` connects to a USB-attached device via ADB, builds a debug APK (no signing required — uses the auto-generated debug keystore), installs it, and starts the app. Frontend changes hot-reload via Vite without reinstalling. Rust changes trigger a full rebuild and reinstall automatically.

Prerequisites:
- Enable Developer Options on the phone (tap Build Number 7 times in Settings → About)
- Enable USB Debugging in Developer Options
- Trust the Mac when prompted on the phone
- Verify device is visible: `adb devices` should list it

This replaces the manual build → sign → install loop entirely for day-to-day testing.

**For release testing: automate signing**

Release APKs require a signed keystore. Set this up once so `pnpm tauri android build` produces a ready-to-install APK without manual steps:

1. Generate a keystore (one-time):
   ```bash
   keytool -genkey -v -keystore fitness-notes.jks -alias fitness-notes \
     -keyalg RSA -keysize 2048 -validity 10000
   ```
   Store it outside the repo (e.g. `~/.android/fitness-notes.jks`).

2. Configure Tauri to use it — in `src-tauri/gen/android/app/build.gradle.kts`, add a `signingConfigs` block:
   ```kotlin
   signingConfigs {
       create("release") {
           storeFile = file(System.getenv("ANDROID_KEYSTORE_PATH") ?: "")
           storePassword = System.getenv("ANDROID_KEYSTORE_PASS") ?: ""
           keyAlias = System.getenv("ANDROID_KEY_ALIAS") ?: ""
           keyPassword = System.getenv("ANDROID_KEY_PASS") ?: ""
       }
   }
   buildTypes {
       release { signingConfig = signingConfigs.getByName("release") }
   }
   ```
   Set the four env vars in your shell profile (not committed to the repo).

3. Add a deploy script at the repo root (`deploy.sh`):
   ```bash
   #!/usr/bin/env bash
   set -e
   pnpm tauri android build
   adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
   echo "Installed."
   ```
   `adb install -r` replaces the existing install without losing data. Make executable: `chmod +x deploy.sh`. Run with `./deploy.sh`.

**Summary**:
- Daily iteration → `pnpm tauri android dev` (USB, debug, HMR)
- Release testing → `./deploy.sh` (signs + installs in one step)
- No more manual signing or copy-paste adb commands

---

### 5. Settings menu

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

### 6. Complete body tracker

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

### 7. Workout templates

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

### 8. Season-wise personal bests

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

### 9. Analysis page

New route `/analysis`. Three sections:

**A. Correlation explorer** — pick two series (any body metric or exercise estimated 1RM), plot on a dual-axis or scatter chart. New command: `get_aligned_series(series_a, series_b, from, to) → Vec<{date, a, b}>` using forward-fill for dates where one series has no data.

**B. Summary statistics** — per body metric: current value, 30/90-day delta, all-time min/max. New command: `get_body_summary_stats() → Vec<{metric, current, delta_30d, delta_90d, min, max}>`.

**C. Trend lines** — linear regression overlay on any time-series chart, computed client-side (least-squares on already-fetched data, no new backend command needed).

Add an analysis icon to the header nav in `src/routes/+layout.svelte`.

---

### 10. Data safety

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
5. **Body metrics overhaul** — is_derived, FitNotes renames, derived metric rows; must land before import
6. **Body measurements import** — needed to bring in historical data before going mobile
7. **Android build/test workflow** — reduce iteration friction
8. **Settings menu** — user profile, dark mode, manual export/restore
9. **Complete body tracker** (graph + PRs)
10. **Workout templates**
11. **Season PRs** — depends on settings
12. **Analysis page** — most complex, last
