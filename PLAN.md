# Plan

## What's built

- **Feed** (`/`): Infinite-scroll daily workout log, drag-to-reorder exercises and sets, PR badges
- **Calendar** (`/calendar`): Month view with activity dots, navigates to feed date
- **Exercise tracker** (`/exercise/[id]/[date]`): Full CRUD for sets, PR tracking, history, graph (estimated 1RM), PRs table (nRM viewer)
- **Body tracker** (`/body/[date]`): Log measurements per metric; derived metrics (BMI, Body Fat (Navy), FFMI (Navy)) computed on the fly from stored inputs, never written to DB; history view. `is_derived` flag in `body_metrics` controls read-only rendering. Derived metric dates reflect the most recently updated input.
- **Settings** (`/settings`): FitNotes exercise CSV import wizard, FitNotes Body Tracker CSV import wizard (resolve unknowns: create/map/skip), delete all data. Preferences/user profile: Sex, height and light/dark mode toggle
- **Light/dark mode**: CSS variables defined; `dark_mode` stored in DB but not yet wired to the UI
- **Exercise/category management** (`/exercises/[date]`): Category → exercise drill-down, full CRUD (create, rename, delete, merge) for both categories and exercises via inline inputs and ⋯ context menus. Merge moves all sets/history to the target and recomputes PRs. Errors surface as toasts.
- **Workout/exercise model refactor**: Migration v4 — `workout_order` added to `workouts`, unique constraints removed from `workouts.date` and `workout_exercises(workout_id, exercise_id)`, `sets` now references `workout_exercise_id` FK with `exercise_id` kept denormalized. Route is `/exercise/[id]/[we_id]` (deviation from plan: `exercise_id` kept in URL so history/graph/prs sub-routes can resolve it via the shared `[id]` layout without an extra lookup). Long-press select mode in DayCard allows merge and delete of `workout_exercise` instances. `reorder_exercises` recomputes PR flags after reorder.
- **Migration infrastructure**: `run_migrations()` in `database.rs` with downgrade guard and per-version functions. WAL mode enabled at startup. Automatic daily backups (last 14 kept) on every launch.
- **Android build & deploy**: App runs on Android via Tauri. Touch targets, DnD, viewport, and file picker verified. Daily iteration via `pnpm tauri android dev` (USB, HMR). Release builds signed via keystore env vars; `deploy.sh` builds and installs in one step.
- **Workout templates** (`/templates/[date]`): Save any workout as a named template (snapshots exercises + sets); apply to any date to pre-populate a new workout. Template CRUD (list, rename, delete) via `SelectList`. Applying a template recomputes PR flags for each affected exercise. `DayCard` shows a paste icon on rest days and a copy icon on workout days. Workout name column (`workouts.name`, migration v7) is pending — apply will name the workout after the template, and `DayCard` will show the name instead of the generic title.

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
    id            integer primary key,
    date          text not null,
    workout_order integer not null default 1,
    name          text                         -- nullable;
)

templates (
    id   integer primary key,
    name text not null unique
)

template_exercises (
    id             integer primary key,
    template_id    integer not null references templates(id),
    exercise_id    integer not null references exercises(id),
    exercise_order integer not null
)

template_sets (
    id                   integer primary key,
    template_id          integer not null references templates(id),
    template_exercise_id integer not null references template_exercises(id),
    set_order            integer not null,
    weight_kg            real not null,
    reps                 integer not null
)

workout_exercises (
    id             integer primary key,
    workout_id     integer not null references workouts(id),
    exercise_id    integer not null references exercises(id),
    exercise_order integer not null
)

sets (
    id                  integer primary key,
    workout_exercise_id integer not null references workout_exercises(id),
    exercise_id         integer not null references exercises(id),  -- denormalized for fast PR queries
    set_order           integer not null,
    weight_kg           real not null,
    reps                integer not null,
    notes               text,
    was_pr_at_time      boolean not null,
    is_current_pr       boolean not null
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
v8: ALTER TABLE user_settings ADD COLUMN season_start TEXT DEFAULT '01-01'
v9: ALTER TABLE user_settings ADD COLUMN use_seasons BOOLEAN DEFAULT true
v10: ALTER TABLE sets ADD COLUMN is_season_pr BOOLEAN DEFAULT false
```

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

### 8. Season-wise personal bests

A season is a 1-year window starting from a user-configured month/day (MM-DD), recurring annually. The current season = from the most recent occurrence of that date to today.

**DB changes** (migrations v8–v10):
```sql
ALTER TABLE user_settings ADD COLUMN season_start TEXT DEFAULT '01-01';
ALTER TABLE sets           ADD COLUMN is_season_pr BOOLEAN DEFAULT false;
ALTER TABLE sets           ADD COLUMN was_season_pr_at_time BOOLEAN DEFAULT false;
```

**Logic**: `is_season_pr` uses the same Pareto-frontier algorithm as `is_current_pr`, but scoped only to sets within season windows, so there can be multiple sets per rep count and exercise id pair with is_season_pr true, but only one per season. Additionally, the was_season_pr_at_time is analogous to was_pr_at_time. Recomputed on every set insert/edit/delete via a new `recompute_season_prs(conn, exercise_id)` helper, or add to recmpute_pr_flags. Note that on insert/edit/delete, only the season that the date is in needs to be recomputed. 

**Commands**:
- `get_season_rep_maxes(exercise_id) → Vec<RepMax>`
- Update `upsert_set` / `delete_set` to also recompute season PRs

**Frontend**:
- PRs page (`/exercise/[id]/prs`): add a second column "This season" alongside "All time" for each rep count row.
- Settings page: season start MM-DD input
- Set rows: Add season pr badges for current and at-time.

---

### 9. Analysis page

New route `/analysis`. Three sections:

**A. Correlation explorer** — pick two series (any body metric or exercise estimated 1RM), plot on a dual-axis or scatter chart. New command: `get_aligned_series(series_a, series_b, from, to) → Vec<{date, a, b}>` using forward-fill for dates where one series has no data.

**B. Summary statistics** — per body metric: current value, 30/90-day delta, all-time min/max. New command: `get_body_summary_stats() → Vec<{metric, current, delta_30d, delta_90d, min, max}>`.

**C. Trend lines** — linear regression overlay on any time-series chart, computed client-side (least-squares on already-fetched data, no new backend command needed).

Add an analysis icon to the header nav in `src/routes/+layout.svelte`.

Possible interesting analysis:
- Pareto frontier (reps vs highest weight for rep count), comparing exercises and season could be interesting, as well as the shape of the curve. 

**Extension**: full statistical modeling for this page is designed in §13 (posterior ribbons, couplings, forecasts). The simple correlation explorer above remains the v1.

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

### 11. Workout templates management

Currently, the workout planning feature is limited to saving a workout as a template, giving it a name, and pasting it on a day where there is no current workout. This feels a bit limiting, as the user would have to modify actual sets and workouts in order to create a new template workout, and there isn't a simple way to see what templates are saved or to edit existing templates. Therefore, a page for managing templates would be handy.

**Layout**
I am imagining something very similar to the daycards, except there would just be a single daycard (date = templates). Differences from day-card: 
- An add button for creating a new template (this should likely also be on the day card for multi-workout support, but that is potential future stuff.)
- A delete button for deleting a template
- Remove the body button.
- Templates sorted alpabetically by name.
- Rename pencil button by the template name 
- Remove the copy/paste buttons.

**New commands** 
- upsert_template_set(id: Option<i64>, template_exercise_id: i64, weight_kg: f64, reps: i64) -> Set
- delete_template_set(id: i64) -> ()
- reorder_template_exercises(ordered_template_exercise_ids) -> ()
- reorder_template_sets(template_exercise_id: i64, ordered_set_ids: Vec<i64>) -> ()
- add_exercise_to_template(id: i64, exercise_id: i64) -> i64
- remove_exercise_from_template(template_exercise_id: i64) -> ()
- get_templates() -> Vec<TemplateWithExercises>
- merge_template_exercises(template_exercise_ids: Vec<i64>) -> ()


**Frontend**
- Create /templates, essentially a modified DayCard with a simple header (back button), and a title "Edit templates" or "Templates"
- Wire up link to /templates from settings, the select template page, and the copy to clipboard button.
- new edit sets route for template sets. For reusing the existing one as much as possible, it can be placed in templates/[id]/[te_id]/

**Notes**
All of the new commands will be pretty much copy-pasted from their real-workout counterparts, the same goes for the frontend additions. I think I am mostly fine with this duplication, especially considering that the templates feature may diverge from the standard exercise tracking mode more in the future (weight/reps can become percentage/reps, rpe/reps, ... for better planning purposes), and I do not want to prematurely figure out good abstractions, as I don't really see them yet. 

---

---

### 12. Publish to the iOS and Android app stores

Current state: both mobile projects are initialized (`src-tauri/gen/android`, `src-tauri/gen/apple`), all Rust mobile targets are installed, and Android release APKs are built + signed via keystore env vars (`deploy.sh`). iOS has never been built or signed (`developmentTeam` is still a placeholder, `ExportOptions.plist` method is `debugging`).

#### Phase 0 — Accounts and prerequisites (start immediately)

- Enroll in Google Play Console ($25 one-time; new personal accounts need identity verification) and the Apple Developer Program ($99/yr; ~1–2 day approval).
- Host a privacy policy (GitHub Pages is fine): all data stored locally on device, nothing collected, shared, or transmitted. Required by both stores.
- Decide the real app name (display name + store listing). Not blocking for builds; "Fitness Notes" stays the placeholder until then.

#### Phase 1 — Android → Play Store (first launch)

- Build AABs for Play: `pnpm tauri android build --aab` (deploy.sh builds APK + AAB; APK stays for device sideloading).
- Back up the release keystore + passwords durably — losing it complicates updates. The env-var signing config is reused as the Play upload key.
- Verify generated `targetSdk` ≥ 35 (Play requirement for new apps) and the manifest app label.
- Play Console setup: store listing (title, descriptions, 512×512 icon, 1024×500 feature graphic, 2+ phone screenshots), content rating questionnaire, Data safety form (declare: no data collected/shared — all local), privacy policy URL.
- Closed test: new personal Play accounts must run **12 opted-in testers continuously for 14 days** before production access is granted. Recruit testers early — longest lead-time item.
- Rollout: internal → closed (satisfies the requirement) → production.

#### Phase 2 — iOS bring-up (in parallel with Phase 1 waiting periods)

- Set real Team ID in `tauri.conf.json` → `bundle.iOS.developmentTeam`.
- `pnpm tauri ios dev` on simulator, then a real device. Big unknown — first-ever iOS run. Expect fixes for: safe areas/notches, keyboard insets, export/restore flow (`tauri-plugin-dialog` `save` is desktop-only → needs share-sheet fallback or platform gating), general touch feel.
- Set `ExportOptions.plist` method to `app-store`, then `pnpm tauri ios build` → upload via Xcode or Transporter.
- App Store Connect: app record (bundle ID `com.vegardbroen.fitness-notes` is valid for iOS), listing (description, keywords, screenshots at required iPhone sizes, age rating, privacy labels — nothing collected), TestFlight, then App Review (Health & Fitness category; no special entitlements needed).

#### Phase 3 — Release mechanics

- Bump `version` in `tauri.conf.json` each release; Android needs an incrementing `versionCode`, iOS an incrementing build number.
- Optional later: GitHub Actions (`tauri-action`) for automated signed mobile builds.

#### Timeline estimate

Android in production ~3–4 weeks out (dominated by the 14-day closed test + listing prep). iOS after that, dominated by first-run device debugging; App Review itself is ~1–2 days once submitted.

---

### 13. Statistical modeling (extends §9 Analysis page)

**Status: designed, not started. Blocked until the in-progress refactor lands.**

#### Design decisions (settled)

- **Joint linear-Gaussian state-space model** over all variables (body metrics + exercise e1RM). Kalman filter + RTS smoother give exact joint posteriors — every variable's history conditions on all data from all variables, both directions in time. Parameters via EM (Shumway–Stoffer), i.e. empirical Bayes: no MCMC, no Python, phone-cheap.
- **Tree factor structure** for exercises — identified by construction (no rotation ambiguity): global strength factor `g_t` → per-movement-pattern factors `f_c,t = λ_c·g_t + η_c` (`Var(η_c)=1`, `λ_c ≥ 0`); each exercise loads only on its pattern factor. Rationale: a rare variant (paused squat) pools information through its pattern factor (low-bar/high-bar squat, leg press) — a better prior than a single global factor. Rank-1-global and free-d variants were considered and dropped for now; free-d may come later behind the same `FactorSpec` interface (PCA → varimax → anchored EM).
- **Static couplings** `β_jk` (exercise ← smoothed body-metric levels), admitted per-coupling via exact marginal likelihood (with/without fit, BIC-penalized). Time-varying couplings are deliberately NOT in the joint model (a time-varying coefficient × a latent state is bilinear — no exact filtering); they live in a separate on-demand diagnostics layer.
- **Variant offsets** `δ` per (exercise, variant) absorb known measurement breaks (gym/machine changes, new bathroom scale) so levels and couplings stay clean — a break is a known, dated, additive instrument change and belongs in the observation equation, not in time-varying coefficients. Ridge-shrunk toward 0, reference variant pinned at 0, user-visible ("Gym X chest press ≈ +4 kg vs reference").
- **RIR**: optional per-set field. `e1RM = w·(1 + (reps + RIR)/30)` when present (Epley assumes failure); lower observation noise for RIR sets. Without RIR the model estimates a *performance index* (strength and effort are confounded — slow effort changes cannot be separated from strength gains); UI wording should reflect that. **PR logic stays on raw (weight, reps)** — RIR is modeling metadata only.
- **Clean data via detection, not policing**: the online filter's standardized innovation flags implausible logs → toast ("looks off — different machine?") → one-tap tag/break-date assignment; offsets learn the jump retroactively.

#### Model

```
State (all random walks, process noise Q·Δt — irregular gaps handled natively, missing days = predict-only):
  ℓ_i,t   body-metric levels (log scale for mass-type metrics)
  ℓ_j,t   exercise levels
  g_t     global strength factor
  f_c,t = λ_c·g_t + η_c ,  Var(η_c) = 1, λ_c ≥ 0        pattern factors, c = 1..C

Observation:
  body metric i:   y_i,t = ℓ_i,t + ε_i,t
  exercise j:      y_j,t = ℓ_j,t + b_j·f_cat(j),t + Σ_k β_jk·ℓ^body_k,t
                         + δ_j,variant(j,t) + ε_j,t
```

- e1RM observation noise `h·(1 + reps/30)²` (extrapolation grows with reps), halved when RIR present. Outlier guard: standardized innovation > 3σ → inflate that observation's noise.
- Estimation: square-root Kalman filter (obs-by-obs updates, `H` diagonal) → RTS smoother + lag-one covariances → closed-form M-step for all variances; regression-on-smoothed-moments for `b_j, λ_c, β_jk, δ`. ~10–50 EM iterations.
- Perf budget: state dim ≈ 80–100 (active exercises only, ≥5 sets in window) → full refit ≲ 2–3 s on a phone; run on app open / debounced on data change, cache results. Documented fallback if old phones struggle: block the filter (body block exact; exercises conditioned on smoothed body/factor paths).
- Diagnostics layer (on demand, separate): two-stage plug-in filters → time-varying `β_jk(t)` paths; windowed partial correlations on smoothed innovations. Doubles as a detector for undeclared breaks (residual level shift aligned with a date → prompt).

#### Data model (provisionally v11–v12 — renumber after seasons v8–v10 land)

```sql
ALTER TABLE workouts          ADD COLUMN tag     TEXT;  -- gym/location context, free text, nullable
ALTER TABLE workout_exercises ADD COLUMN variant TEXT;  -- per-entry "different machine" override, nullable
ALTER TABLE exercises         ADD COLUMN pattern TEXT;  -- optional movement-pattern group (factor group)
ALTER TABLE sets              ADD COLUMN rir     INTEGER;             -- nullable, per set
ALTER TABLE user_settings     ADD COLUMN track_rir INTEGER DEFAULT 0;
```

- Effective variant key: `coalesce(workout_exercises.variant, workouts.tag, 'default')` — offsets attach to (exercise, variant) pairs. No new tables or FKs; tag autocomplete = `SELECT DISTINCT tag FROM workouts WHERE tag IS NOT NULL`.
- `estimate_1rm(weight, reps, rir)` centralized in one helper; the existing exercise graph adopts it.

#### UX

- Settings: "Track RIR" toggle → optional RIR stepper on set rows (remembers last value per exercise).
- Workout tag on the day card / workout header: sticky (defaults to last used), autocomplete.
- "Different machine…" in the exercise-entry ⋯ menu (variant override, pre-filled with inherited tag).
- Pattern field in exercise edit (optional, suggested from existing values).
- Analysis page additions: posterior ribbons (smoothed ± 2σ + forecast), couplings table (β, loadings), offsets view, time-varying diagnostics toggle, innovation-flag toast flow.

#### Stats module (`src-tauri/src/stats/`, only new dep: `faer` or `nalgebra`)

- `filter.rs` — square-root Kalman, obs-by-obs updates, generic over the observation matrix
- `smoother.rs` — RTS smoother + lag-one covariances
- `fit.rs` — EM loop (closed-form variance updates, moments regression for loadings/couplings/offsets)
- `factors.rs` — `FactorSpec` (factor list + loading pattern; tree now, free-d later)
- `diagnostics.rs` — two-stage β_t paths, windowed partial correlations

New commands: `posterior_series`, `forecast`, `get_couplings`, `get_offsets`, `coupling_path`, `get_innovation_flags`.

#### Validation

- Synthetic-data recovery of Θ (incl. a simulated offset jump and tree factors)
- Held-out forecasts beat naive last-value baseline, especially for sparse variants (the tree model's success criterion)
- Innovation whiteness (no residual autocorrelation ⇒ trend structure adequate)
- Unit tests: `estimate_1rm`, variant-key resolution

#### Implementation order

0. **Blocked: in-progress refactor**
1. Migrations + `estimate_1rm` + RIR/tag/variant/pattern UX — ships value before any model
2. Stats core (filter/smoother/EM/FactorSpec) + per-variable posteriors powering the existing exercise/body graphs
3. Joint tree model: factors, offsets, β admission via marginal likelihood; commands + analysis UI
4. Diagnostics layer + break detector
5. Clean-data anomaly loop (innovation toast → tag/break-date repair)
6. Later: free-d factors behind `FactorSpec`

---

## Implementation order

1. ~~**Migration infrastructure**~~ ✓ done
2. ~~**WAL mode + automatic backups**~~ ✓ done
3. ~~**Create and manage exercises**~~ ✓ done
4. ~~**Android build**~~ ✓ done
5. ~~**Body metrics overhaul**~~ ✓ done — is_derived, FitNotes renames, on-the-fly derived metrics, write guard in upsert
6. ~~**Body measurements import**~~ ✓ done
7. ~~**Android build/test workflow**~~ ✓ done — `tauri android dev` for iteration, `deploy.sh` for release
8. ~~**Workout/exercise model refactor**~~ ✓ done — workout_exercise_id, exercise repetition, merge/delete via select mode, backend tests
9. ~~**Settings menu**~~ ✓ done — user profile, dark mode, manual export/restore
10. ~~**Workout templates**~~ ✓ done — save/apply/rename/delete; PR recomputation on apply; workout name column (v7) pending
11. **Complete body tracker** (graph + PRs)
12. **Season PRs** — depends on settings
13. **Analysis page** — most complex, last
14. **Publish to app stores** — Phase 0 (accounts/privacy policy) anytime; Android first (section 12), iOS bring-up in parallel
15. **Statistical modeling** (section 13) — blocked on the in-progress refactor; phase 1 (migrations + RIR/tag/variant/pattern UX) ships value before the model itself

---

## Deferred: multiple workouts per day

The schema refactor (section 5) removes the `UNIQUE` constraint on `workouts.date` and adds a `workout_order` column, so the database already supports multiple workouts per day. This is intentionally not exposed in the UI yet.

When/if the time comes, the remaining work is frontend-only:

- **Feed** (`src/lib/DayCard.svelte`): group exercises under named workout blocks ("Workout 1", "Workout 2"); add an "Add workout" button per day that calls a new `create_workout(date) → workout_id` command
- **`create_workout` command** (`src-tauri/src/commands/workouts.rs`): `INSERT INTO workouts (date, workout_order) VALUES (?1, (SELECT COALESCE(MAX(workout_order), 0) + 1 FROM workouts WHERE date = ?1))`
- **`reorder_exercises`**: already takes `workout_id` after the refactor, so multi-workout reordering works without further backend changes
- No migration needed — the schema is already correct after v4
