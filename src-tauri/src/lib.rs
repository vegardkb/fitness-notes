use serde::Serialize;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_workout_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<ExerciseWithSets>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name, c.name, we.exercise_order,
                    s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN categories c ON e.category_id = c.id
             JOIN sets s ON s.workout_id = w.id AND s.exercise_id = e.id
             WHERE w.date = ?1
             ORDER BY we.exercise_order, s.set_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, i64>(0)?,            // exercise_id
                row.get::<_, String>(1)?,         // exercise_name
                row.get::<_, String>(2)?,         // category
                row.get::<_, i64>(3)?,            // exercise_order
                row.get::<_, i64>(4)?,            // set id
                row.get::<_, i64>(5)?,            // set_order
                row.get::<_, f64>(6)?,            // weight_kg
                row.get::<_, i64>(7)?,            // reps
                row.get::<_, Option<String>>(8)?, // notes
                row.get::<_, bool>(9)?,           // was_pr_at_time
                row.get::<_, bool>(10)?,          // is_current_pr
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<ExerciseWithSets> = Vec::new();
    for row in rows {
        let (
            exercise_id,
            exercise_name,
            category,
            exercise_order,
            set_id,
            set_order,
            weight_kg,
            reps,
            notes,
            was_pr_at_time,
            is_current_pr,
        ) = row.map_err(|e| e.to_string())?;

        let set = Set {
            id: set_id,
            set_order,
            weight_kg,
            reps,
            notes,
            was_pr_at_time,
            is_current_pr,
        };

        match result.last_mut() {
            Some(last) if last.exercise_id == exercise_id => last.sets.push(set),
            _ => result.push(ExerciseWithSets {
                exercise_id,
                exercise_name,
                category,
                exercise_order,
                sets: vec![set],
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
fn list_exercise_categories(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<String>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT name FROM categories")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;

    let mut result: Vec<String> = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }

    Ok(result)
}

#[tauri::command]
fn list_exercises_in_category(
    category: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Exercise>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name FROM exercises e
             JOIN categories c ON e.category_id = c.id
             WHERE c.name = ?1
             ORDER BY e.name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![category], |row| {
            Ok(Exercise { id: row.get(0)?, name: row.get(1)? })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
fn get_exercise(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Exercise, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name FROM exercises WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok(Exercise { id: row.get(0)?, name: row.get(1)? }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_set(
    date: &str,
    exercise_id: i64,
    weight_kg: f64,
    reps: i64,
    notes: Option<String>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Set, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    // Find or create workout for this date
    conn.execute(
        "INSERT OR IGNORE INTO workouts (date) VALUES (?1)",
        rusqlite::params![date],
    )
    .map_err(|e| e.to_string())?;
    let workout_id: i64 = conn
        .query_row(
            "SELECT id FROM workouts WHERE date = ?1",
            rusqlite::params![date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Find or create workout_exercises entry
    let next_exercise_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(exercise_order), 0) + 1 FROM workout_exercises WHERE workout_id = ?1",
            rusqlite::params![workout_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO workout_exercises (workout_id, exercise_id, exercise_order) VALUES (?1, ?2, ?3)",
        rusqlite::params![workout_id, exercise_id, next_exercise_order],
    )
    .map_err(|e| e.to_string())?;

    // Get next set_order for this exercise in this workout
    let set_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(set_order), 0) + 1 FROM sets WHERE workout_id = ?1 AND exercise_id = ?2",
            rusqlite::params![workout_id, exercise_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Compute was_pr_at_time: is this set dominated by any prior set for this exercise?
    let dominated: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sets s
             JOIN workouts w ON s.workout_id = w.id
             WHERE s.exercise_id = ?1
               AND (w.date < ?2 OR (w.date = ?2 AND s.set_order < ?3))
               AND s.weight_kg >= ?4
               AND s.reps >= ?5",
            rusqlite::params![exercise_id, date, set_order, weight_kg, reps],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let was_pr_at_time = !dominated;

    // Insert the set
    conn.execute(
        "INSERT INTO sets (workout_id, exercise_id, set_order, weight_kg, reps, notes, was_pr_at_time, is_current_pr)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
        rusqlite::params![workout_id, exercise_id, set_order, weight_kg, reps, notes, was_pr_at_time],
    )
    .map_err(|e| e.to_string())?;
    let set_id = conn.last_insert_rowid();

    // Recompute is_current_pr for all sets of this exercise
    conn.execute(
        "UPDATE sets SET is_current_pr = NOT EXISTS (
             SELECT 1 FROM sets s2
             WHERE s2.exercise_id = sets.exercise_id
               AND s2.id != sets.id
               AND s2.weight_kg >= sets.weight_kg
               AND s2.reps >= sets.reps
         )
         WHERE exercise_id = ?1",
        rusqlite::params![exercise_id],
    )
    .map_err(|e| e.to_string())?;

    let is_current_pr: bool = conn
        .query_row(
            "SELECT is_current_pr FROM sets WHERE id = ?1",
            rusqlite::params![set_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(Set { id: set_id, set_order, weight_kg, reps, notes, was_pr_at_time, is_current_pr })
}

#[derive(Serialize)]
struct Exercise {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
struct ExerciseWithSets {
    pub exercise_id: i64,
    pub exercise_name: String,
    pub category: String,
    pub exercise_order: i64,
    pub sets: Vec<Set>,
}

#[derive(Serialize)]
struct Set {
    pub id: i64,
    pub set_order: i64,
    pub reps: i64,
    pub weight_kg: f64,
    pub notes: Option<String>,
    pub was_pr_at_time: bool,
    pub is_current_pr: bool,
}

fn initialize_db(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    let conn = rusqlite::Connection::open(app_data_dir.join("fitness_notes.db"))?;

    create_tables(&conn)?;
    app.manage(std::sync::Mutex::new(conn));
    Ok(())
}

fn create_tables(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
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
             date text not null unique,
             weight_kg real,
             body_fat real,
             waist_cm real,
             neck_cm real,
             chest_cm real,
             shoulder_cm real,
             hips_cm real,
             left_arm_cm real,
             right_arm_cm real,
             left_thigh_cm real,
             right_thigh_cm real,
             left_calf_cm real,
             right_calf_cm real
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists user_settings (
             id integer primary key check (id = 1),
             height_cm integer not null,
             unit text not null default 'kg',
             estimate_body_fat boolean not null default true
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            initialize_db(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_workout_for_date,
            list_exercise_categories,
            list_exercises_in_category,
            get_exercise,
            add_set,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
