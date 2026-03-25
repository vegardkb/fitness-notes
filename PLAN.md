# Plan
This document will outline the plan for developing a fitness note taking app.

## Features
### Core
- Add, edit and view sets, and if sets are personal records (current or at the time).
- Different views
  - Exercise view (see/edit the sets performed on an exercise on a particular day)
    - Secondary views:
      - History. Show previous days/sets of this exercise
      - Graph. Show interactive graph of date vs metrics (estimated 1rm, PRs, volume, etc)
  - Day view (see/edit the exercises performed on a particular day. Jump to exercise view)
  - Calendar view (see what days exercise was performed/jump to day view)
- Import data tool
  - Should at least work for fitnotes export
- Export data tool
  - To csv
- Body tracker
  - weight, body fat (manual entry or estimate using navy body fat if waist and neck measurements are available), circumference measurements
  - graph view for body tracker metrics

### Interesting ideas
- I think modeling the relationship between different metrics could be interesting, weight vs estimated 1rm for instance.
- Not sure if this is a good concept for strength training, but in cardio world a cumulative load measurement is often useful. Essentially a convolution of daily load with an exponential kernel. Though the daily load is maybe difficult to define for strength, and may be the wrong concept.

---

## Implementation Plan

### Tech stack additions
- **Database**: SQLite via `rusqlite` in the Rust backend. All DB access goes through typed Tauri commands — no SQL is issued from the frontend. The DB file lives in the Tauri app data directory (`$APPDATA/fitness-notes/fitness-notes.db`), resolved at runtime via `tauri::path::app_data_dir`.
- **Migrations**: Embedded SQL migration files applied at startup using `rusqlite_migration` (or hand-rolled version table). Migration files live in `src-tauri/migrations/`.
- **Graphing**: A lightweight JS charting library (e.g. Chart.js or uPlot) for the exercise and body tracker graphs.
- **Routing**: SvelteKit file-based routing. All routes are pre-rendered (static).

### Database Schema

```sql
-- Exercise categories (e.g. "Push", "Pull", "Legs")
CREATE TABLE categories (
  id    INTEGER PRIMARY KEY,
  name  TEXT NOT NULL UNIQUE
);
-- Default categories seeded on first run:
-- Abs, Back, Biceps, Chest, Legs, Shoulders, Triceps

-- Exercises (the movement, not a specific session)
CREATE TABLE exercises (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  category_id INTEGER NOT NULL,
  FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- A workout is a collection of sets performed on a single date
CREATE TABLE workouts (
  id    INTEGER PRIMARY KEY,
  date  TEXT NOT NULL UNIQUE  -- ISO 8601 "YYYY-MM-DD"
);

-- Tracks which exercises appear in a workout and their display order
CREATE TABLE workout_exercises (
  id              INTEGER PRIMARY KEY,
  workout_id      INTEGER NOT NULL,
  exercise_id     INTEGER NOT NULL,
  exercise_order  INTEGER NOT NULL,
  FOREIGN KEY (workout_id) REFERENCES workouts(id),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id),
  UNIQUE (workout_id, exercise_id)
);

-- Individual sets within a workout
CREATE TABLE sets (
  id              INTEGER PRIMARY KEY,
  workout_id      INTEGER NOT NULL,
  exercise_id     INTEGER NOT NULL,
  set_order       INTEGER NOT NULL,  -- ordering within a workout/exercise
  weight_kg       REAL NOT NULL,     -- always stored in kg; convert to display unit in the frontend
  reps            INTEGER NOT NULL,
  notes           TEXT,
  was_pr_at_time  BOOLEAN NOT NULL,
  is_current_pr   BOOLEAN NOT NULL,
  FOREIGN KEY (workout_id) REFERENCES workouts(id),
  FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);

-- Body measurements (all optional except date)
CREATE TABLE body_measurements (
  id             INTEGER PRIMARY KEY,
  date           TEXT NOT NULL UNIQUE,  -- ISO 8601 "YYYY-MM-DD"
  weight_kg      REAL,                  -- always stored in kg
  body_fat       REAL,                  -- percentage, manually entered
  waist_cm       REAL,
  neck_cm        REAL,
  chest_cm       REAL,
  shoulder_cm    REAL,
  hips_cm        REAL,
  left_arm_cm    REAL,
  right_arm_cm   REAL,
  left_thigh_cm  REAL,
  right_thigh_cm REAL,
  left_calf_cm   REAL,
  right_calf_cm  REAL
);

-- User settings (single row, enforced by CHECK)
CREATE TABLE user_settings (
  id                  INTEGER PRIMARY KEY CHECK (id = 1),
  height_cm           REAL,     -- used for navy body fat formula
  unit               TEXT NOT NULL DEFAULT 'kg',  -- 'kg' or 'lb'
  estimate_body_fat   BOOLEAN NOT NULL DEFAULT true
);
```

**PR definition**: A set `(weight=x, reps=n)` is a PR if no other set for that exercise has `weight >= x AND reps >= n`. This is the Pareto frontier of the (weight, reps) space — a 100kg×5 and 80kg×10 can both be PRs simultaneously as neither dominates the other.

Two flags are stored per set:
- `was_pr_at_time`: was this set on the Pareto frontier of all sets for this exercise logged *up to and including this set's date*? Set at log/import time; never cleared (a historical PR stays marked even after being surpassed).
- `is_current_pr`: is this set on the Pareto frontier of *all* sets ever logged for this exercise? Recomputed for all sets of an exercise whenever any set is added, edited, or deleted.

### Frontend Routes

```
/                          → Calendar view
/day/[date]                → Day view (YYYY-MM-DD)
/exercise/[id]/[date]      → Exercise view for a specific date (YYYY-MM-DD)
/exercise/[id]/history     → All sets across all dates
/exercise/[id]/graph       → Interactive graph
/body                      → Body tracker log
/body/graph                → Body tracker graphs
/import                    → Import tool
/export                    → Export tool
```

Unit preference (kg/lb) is stored in `user_settings` in the DB (single source of truth). Current date and other ephemeral UI state live in a Svelte store.

**SvelteKit prerendering**: the global layout sets `prerender = true`, but dynamic routes cannot be pre-rendered at build time. Each dynamic route (`/day/[date]`, `/exercise/[id]/[date]`, etc.) must set `export const prerender = false`. Static routes (`/`, `/import`, `/export`, `/body`, `/body/graph`) can keep prerendering enabled.

### Rust Commands (src-tauri/src/lib.rs and submodules)

Organize into modules: `db`, `workouts`, `exercises`, `sets`, `body`, `import_export`.

Key commands to expose via `#[tauri::command]`:

| Command | Description |
|---|---|
| `get_workouts_with_activity` | Returns dates with workout data (for calendar) |
| `get_workout_for_date` | Returns all exercises + sets for a date |
| `upsert_set` | Add or edit a set; recomputes `is_current_pr` for all sets of that exercise; sets `was_pr_at_time` on new sets |
| `delete_set` | Remove a set |
| `get_exercise_history` | All sets for an exercise, grouped by date |
| `get_exercise_graph_data` | Time series: date → estimated 1RM / volume / max weight |
| `list_exercise_categories` | All categories |
| `list_exercises_in_category` | All exercises (for autocomplete) |
| `create_exercise` | Add new exercise |
| `get_body_log` | All body measurement entries |
| `upsert_body_measurement` | Add or edit body measurement; computes navy BF% if applicable |
| `get_body_graph_data` | Time series for a given body metric |
| `import_fitnotes` | Parses FitNotes CSV, inserts data, recomputes all PRs |
| `export_csv` | Streams all sets data as CSV via file dialog |

### PR Recomputation

A set `(x, n)` is dominated by another set `(x', n')` if `x' >= x AND n' >= n` (where the two sets are distinct). A set is a PR if and only if it is not dominated by any other set for that exercise.

**`is_current_pr`** — recomputed for all sets of an exercise after any insert/edit/delete:

```
for each set S for this exercise:
    S.is_current_pr = true
    for each other set T for this exercise:
        if T.weight >= S.weight AND T.reps >= S.reps:
            S.is_current_pr = false
            break
```

This is O(n²) per exercise but n is small (hundreds of sets at most) so it's fine.

**`was_pr_at_time`** — computed once at insert time (or in chronological order during import). For a newly logged set `(x, n)` on date `d`:

```
was_pr_at_time = true
for each prior set T (date < d, or same date with earlier set_order):
    if T.weight >= x AND T.reps >= n:
        was_pr_at_time = false
        break
```

`was_pr_at_time` is **never cleared** — once a set was a PR at the time it was performed, that historical fact is preserved even after being surpassed. However, editing any set's weight or reps can affect the `was_pr_at_time` of all chronologically later sets for that exercise (the edited set may now block or unblock them). Therefore, whenever a set is edited or deleted, `was_pr_at_time` must be fully recomputed for all sets of that exercise in chronological order, identical to the import algorithm.

### Body Fat: Navy Formula

Estimated BF% (men) = `86.010 × log10(waist_cm - neck_cm) - 70.041 × log10(height_cm) + 36.76`

`height_cm` comes from `user_settings`, not from the measurement row. Computed in Rust when `waist_cm`, `neck_cm`, and `user_settings.height_cm` are all present and `body_fat` is not manually set. If manually entered, the manual value takes precedence.

### Import: FitNotes CSV Format

FitNotes exports a CSV with columns: `Date, Exercise, Category, Weight (kg), Reps, Distance, Distance Unit, Time`. Import should:
1. Find or create a row in `categories` for each unique `Category` value.
2. Find or create a row in `exercises` for each unique `Exercise` name, linked to the category.
3. Find or create a workout for each date.
3. Insert sets, preserving original order.
4. Recompute PRs for all imported exercises after bulk insert.

### Export CSV Format

```
date, exercise, category, set_order, weight_kg, reps, was_pr_at_time, is_current_pr, notes
```

### Phases

**Phase 1 – Foundation**
- Set up `rusqlite` with SQLite, migrations, DB initialization
- Day view: list exercises, add/edit/delete sets, exercise autocomplete
- Basic exercise creation

**Phase 2 – Navigation**
- Calendar view showing days with workouts
- Exercise view with today's sets
- Routing between views

**Phase 3 – Exercise insights**
- Exercise history view
- Exercise graph (estimated 1RM over time, volume)
- PR badges on sets

**Phase 4 – Import / Export**
- FitNotes CSV import
- CSV export

**Phase 5 – Body tracker**
- Body measurement log (add/edit entries)
- Navy formula BF% estimation
- Body tracker graphs

**Phase 6 – Polish**
- Unit toggle (kg/lb)
- Keyboard navigation
- Explore: cumulative load metric, cross-metric correlation graphs
