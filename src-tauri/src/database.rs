use crate::models::{Settings, Sex, WeightUnit};

const SCHEMA_VERSION: u32 = 3;

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), String> {
    let current = conn
        .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
        .map_err(|e| e.to_string())?;

    if current > SCHEMA_VERSION {
        return Err("DB is newer than this version of the app".into());
    }

    if current < 1 && 1 <= SCHEMA_VERSION {
        migrate_1(conn).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA user_version = 1")
            .map_err(|e| e.to_string())?;
    }

    if current < 2 && 2 <= SCHEMA_VERSION {
        migrate_2(conn).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA user_version = 2")
            .map_err(|e| e.to_string())?;
    }

    if current < 3 && 3 <= SCHEMA_VERSION {
        migrate_3(conn).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA user_version = 3")
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn migrate_1(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "create table if not exists exercises (
             id integer primary key,
             name text not null unique,
             category_id integer not null,
             foreign key (category_id) references categories(id)
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists categories (
             id integer primary key,
             name text not null unique
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists workouts (
             id integer primary key,
             date text not null unique
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists workout_exercises (
             id             integer primary key,
             workout_id     integer not null,
             exercise_id    integer not null,
             exercise_order integer not null,
             foreign key (workout_id) references workouts(id),
             foreign key (exercise_id) references exercises(id),
             unique (workout_id, exercise_id)
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists sets (
             id integer primary key,
             workout_id integer not null,
             exercise_id integer not null,
             set_order integer not null,
             weight_kg real not null,
             reps integer not null,
             notes text,
             was_pr_at_time boolean not null,
             is_current_pr boolean not null,
             foreign key (workout_id) references workouts(id),
             foreign key (exercise_id) references exercises(id)
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists body_measurements (
             id integer primary key,
             date text not null,
             value real not null,
             measure_id integer not null,
             foreign key (measure_id) references body_metrics(id)
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists body_metrics (
             id integer primary key,
             name text not null unique,
             unit text not null
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists user_settings (
             id integer primary key check (id = 1),
             height_cm integer not null default 178,
             unit text not null default 'kg',
             estimate_body_fat boolean not null default true,
             dark_mode boolean not null default true,
             sex text not null default 'male'
         )",
        (),
    )?;
    conn.execute_batch(
        "INSERT OR IGNORE INTO categories (name) VALUES
             ('Abs'),
             ('Back'),
             ('Biceps'),
             ('Chest'),
             ('Legs'),
             ('Shoulders'),
             ('Triceps');

        INSERT OR IGNORE INTO user_settings (id, height_cm, unit, estimate_body_fat, dark_mode, sex)
        VALUES (1, 178, 'kg', true, true, 'male');


         INSERT OR IGNORE INTO body_metrics (name, unit) VALUES
             ('Weight',     'kg'),
             ('Body Fat',   '%'),
             ('Waist',      'cm'),
             ('Hip',        'cm'),
             ('Neck',       'cm'),
             ('Chest',      'cm'),
             ('Shoulder',   'cm'),
             ('Arm',        'cm'),
             ('Thigh',      'cm'),
             ('Calf',       'cm'),
             ('FFMI',       'kg/m^2');

        UPDATE body_metrics SET unit = 'kg/m^2' WHERE name = 'FFMI';

         INSERT OR IGNORE INTO exercises (name, category_id) VALUES
             -- Abs
             ('Crunch',                (SELECT id FROM categories WHERE name = 'Abs')),
             ('Sit-Up',                (SELECT id FROM categories WHERE name = 'Abs')),
             ('Leg Raise',             (SELECT id FROM categories WHERE name = 'Abs')),
             ('Hanging Leg Raise',     (SELECT id FROM categories WHERE name = 'Abs')),
             ('Plank',                 (SELECT id FROM categories WHERE name = 'Abs')),
             ('Cable Crunch',          (SELECT id FROM categories WHERE name = 'Abs')),
             ('Ab Wheel Rollout',      (SELECT id FROM categories WHERE name = 'Abs')),
             ('Russian Twist',         (SELECT id FROM categories WHERE name = 'Abs')),
             -- Back
             ('Deadlift',              (SELECT id FROM categories WHERE name = 'Back')),
             ('Barbell Row',           (SELECT id FROM categories WHERE name = 'Back')),
             ('T-Bar Row',             (SELECT id FROM categories WHERE name = 'Back')),
             ('Dumbbell Row',          (SELECT id FROM categories WHERE name = 'Back')),
             ('Pull-Up',               (SELECT id FROM categories WHERE name = 'Back')),
             ('Chin-Up',               (SELECT id FROM categories WHERE name = 'Back')),
             ('Lat Pulldown',          (SELECT id FROM categories WHERE name = 'Back')),
             ('Seated Cable Row',      (SELECT id FROM categories WHERE name = 'Back')),
             ('Face Pull',             (SELECT id FROM categories WHERE name = 'Back')),
             -- Biceps
             ('Barbell Curl',          (SELECT id FROM categories WHERE name = 'Biceps')),
             ('EZ-Bar Curl',           (SELECT id FROM categories WHERE name = 'Biceps')),
             ('Dumbbell Curl',         (SELECT id FROM categories WHERE name = 'Biceps')),
             ('Hammer Curl',           (SELECT id FROM categories WHERE name = 'Biceps')),
             ('Incline Dumbbell Curl', (SELECT id FROM categories WHERE name = 'Biceps')),
             ('Concentration Curl',    (SELECT id FROM categories WHERE name = 'Biceps')),
             ('Cable Curl',            (SELECT id FROM categories WHERE name = 'Biceps')),
             ('Preacher Curl',         (SELECT id FROM categories WHERE name = 'Biceps')),
             -- Chest
             ('Bench Press',           (SELECT id FROM categories WHERE name = 'Chest')),
             ('Incline Bench Press',   (SELECT id FROM categories WHERE name = 'Chest')),
             ('Decline Bench Press',   (SELECT id FROM categories WHERE name = 'Chest')),
             ('Dumbbell Bench Press',  (SELECT id FROM categories WHERE name = 'Chest')),
             ('Incline Dumbbell Press',(SELECT id FROM categories WHERE name = 'Chest')),
             ('Dumbbell Fly',          (SELECT id FROM categories WHERE name = 'Chest')),
             ('Cable Fly',             (SELECT id FROM categories WHERE name = 'Chest')),
             ('Pec Deck',              (SELECT id FROM categories WHERE name = 'Chest')),
             ('Push-Up',               (SELECT id FROM categories WHERE name = 'Chest')),
             ('Dip',                   (SELECT id FROM categories WHERE name = 'Chest')),
             -- Legs
             ('Squat',                 (SELECT id FROM categories WHERE name = 'Legs')),
             ('Front Squat',           (SELECT id FROM categories WHERE name = 'Legs')),
             ('Romanian Deadlift',     (SELECT id FROM categories WHERE name = 'Legs')),
             ('Hip Thrust',            (SELECT id FROM categories WHERE name = 'Legs')),
             ('Barbell Lunge',         (SELECT id FROM categories WHERE name = 'Legs')),
             ('Goblet Squat',          (SELECT id FROM categories WHERE name = 'Legs')),
             ('Dumbbell Lunge',        (SELECT id FROM categories WHERE name = 'Legs')),
             ('Bulgarian Split Squat', (SELECT id FROM categories WHERE name = 'Legs')),
             ('Leg Press',             (SELECT id FROM categories WHERE name = 'Legs')),
             ('Leg Extension',         (SELECT id FROM categories WHERE name = 'Legs')),
             ('Leg Curl',              (SELECT id FROM categories WHERE name = 'Legs')),
             ('Calf Raise',            (SELECT id FROM categories WHERE name = 'Legs')),
             ('Hack Squat',            (SELECT id FROM categories WHERE name = 'Legs')),
             ('Nordic Curl',           (SELECT id FROM categories WHERE name = 'Legs')),
             -- Shoulders
             ('Overhead Press',        (SELECT id FROM categories WHERE name = 'Shoulders')),
             ('Dumbbell Shoulder Press',(SELECT id FROM categories WHERE name = 'Shoulders')),
             ('Lateral Raise',         (SELECT id FROM categories WHERE name = 'Shoulders')),
             ('Front Raise',           (SELECT id FROM categories WHERE name = 'Shoulders')),
             ('Rear Delt Fly',         (SELECT id FROM categories WHERE name = 'Shoulders')),
             ('Cable Lateral Raise',   (SELECT id FROM categories WHERE name = 'Shoulders')),
             ('Arnold Press',          (SELECT id FROM categories WHERE name = 'Shoulders')),
             -- Triceps
             ('Close-Grip Bench Press',(SELECT id FROM categories WHERE name = 'Triceps')),
             ('Skull Crusher',         (SELECT id FROM categories WHERE name = 'Triceps')),
             ('Tricep Pushdown',       (SELECT id FROM categories WHERE name = 'Triceps')),
             ('Overhead Tricep Extension',(SELECT id FROM categories WHERE name = 'Triceps')),
             ('Tricep Kickback',       (SELECT id FROM categories WHERE name = 'Triceps')),
             ('Tricep Dip',            (SELECT id FROM categories WHERE name = 'Triceps')),
             ('Diamond Push-Up',       (SELECT id FROM categories WHERE name = 'Triceps'));",
    )?;
    Ok(())
}

fn migrate_2(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "ALTER TABLE body_metrics ADD COLUMN is_derived BOOLEAN NOT NULL DEFAULT false;

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
        UPDATE body_metrics SET name = 'Body Fat (Navy)', is_derived = true WHERE name = 'Body Fat';
        UPDATE body_metrics SET name = 'FFMI (Navy)', is_derived = true WHERE name = 'FFMI';
        INSERT OR IGNORE INTO body_metrics (name, unit, is_derived) VALUES ('BMI', 'kg/m²', true);",
    )?;
    Ok(())
}

fn migrate_3(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "DELETE FROM body_measurements WHERE measure_id IN (SELECT id FROM body_metrics WHERE is_derived = true);"
    )?;
    Ok(())
}

// Recomputes both PR flags for all sets of an exercise.
// was_pr_at_time: full chronological pass (Pareto frontier of prior sets).
// is_current_pr: SQL update checking global dominance.
pub fn recompute_pr_flags(conn: &rusqlite::Connection, exercise_id: i64) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.weight_kg, s.reps FROM sets s
             JOIN workouts w ON s.workout_id = w.id
             WHERE s.exercise_id = ?1
             ORDER BY w.date ASC, s.set_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, f64, i64)> = stmt
        .query_map(rusqlite::params![exercise_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut seen: Vec<(f64, i64)> = Vec::new();
    for (id, weight, reps) in &rows {
        let was_pr = !seen.iter().any(|(w, r)| w >= weight && r >= reps);
        conn.execute(
            "UPDATE sets SET was_pr_at_time = ?1 WHERE id = ?2",
            rusqlite::params![was_pr, id],
        )
        .map_err(|e| e.to_string())?;
        seen.push((*weight, *reps));
    }

    // A set is a current PR if no other set is strictly better in at least one dimension
    // (or equal in both but logged earlier). This ensures that among identical sets,
    // only the chronologically first one is marked as a current PR.
    conn.execute(
        "UPDATE sets SET is_current_pr = NOT EXISTS (
             SELECT 1 FROM sets s2
             JOIN workouts w2 ON s2.workout_id = w2.id
             JOIN workouts w1 ON sets.workout_id = w1.id
             WHERE s2.exercise_id = sets.exercise_id
               AND s2.id != sets.id
               AND s2.weight_kg >= sets.weight_kg
               AND s2.reps >= sets.reps
               AND (
                   s2.weight_kg > sets.weight_kg
                   OR s2.reps > sets.reps
                   OR w2.date < w1.date
                   OR (w2.date = w1.date AND s2.set_order < sets.set_order)
               )
         )
         WHERE exercise_id = ?1",
        rusqlite::params![exercise_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_settings(conn: &rusqlite::Connection) -> Result<Settings, String> {
    let mut stmt = conn
        .prepare("SELECT us.height_cm, us.sex, us.dark_mode, us.unit FROM user_settings us")
        .map_err(|e| e.to_string())?;

    let mut row = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let r = row.next();
    match r {
        Some(Ok((height, sex, dark_mode, unit))) => Ok(Settings {
            height,
            sex: Sex::from(sex),
            dark_mode,
            unit: WeightUnit::from(unit),
        }),
        Some(Err(e)) => Err(e.to_string()),
        None => Err("No settings found".to_string()),
    }
}
