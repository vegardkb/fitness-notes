use crate::models::{DayWorkout, ExerciseWithSets, Set};
use rusqlite::OptionalExtension;

#[tauri::command]
pub fn get_workout_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<ExerciseWithSets>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name, c.name, we.exercise_order, we.id,
                    s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN categories c ON e.category_id = c.id
             JOIN sets s ON s.workout_exercise_id = we.id
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
                row.get::<_, i64>(4)?,            // workout_exercise_id
                row.get::<_, i64>(5)?,            // set id
                row.get::<_, i64>(6)?,            // set_order
                row.get::<_, f64>(7)?,            // weight_kg
                row.get::<_, i64>(8)?,            // reps
                row.get::<_, Option<String>>(9)?, // notes
                row.get::<_, bool>(10)?,          // was_pr_at_time
                row.get::<_, bool>(11)?,          // is_current_pr
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
            workout_exercise_id,
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
                workout_exercise_id,
                exercise_order,
                sets: vec![set],
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_workouts_for_range(
    from_date: &str,
    to_date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DayWorkout>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT w.date, e.id, e.name, c.name, we.exercise_order, we.id,
                    s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN categories c ON e.category_id = c.id
             JOIN sets s ON s.workout_exercise_id = we.id
             WHERE w.date BETWEEN ?1 AND ?2
             ORDER BY w.date DESC, we.exercise_order, s.set_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![from_date, to_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, f64>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, bool>(11)?,
                row.get::<_, bool>(12)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<DayWorkout> = Vec::new();
    for row in rows {
        let (
            date,
            exercise_id,
            exercise_name,
            category,
            exercise_order,
            workout_exercise_id,
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

        let day = match result.last_mut() {
            Some(d) if d.date == date => d,
            _ => {
                result.push(DayWorkout {
                    date: date.clone(),
                    exercises: Vec::new(),
                });
                result.last_mut().unwrap()
            }
        };

        match day.exercises.last_mut() {
            Some(ex) if ex.exercise_id == exercise_id => ex.sets.push(set),
            _ => day.exercises.push(ExerciseWithSets {
                exercise_id,
                exercise_name,
                category,
                workout_exercise_id,
                exercise_order,
                sets: vec![set],
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_active_dates(
    year: i32,
    month: u32,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<String>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let prefix = format!("{}-{:02}", year, month);
    let mut stmt = conn
        .prepare("SELECT DISTINCT date FROM workouts WHERE date LIKE ?1 ORDER BY date")
        .map_err(|e| e.to_string())?;
    let dates = stmt
        .query_map(rusqlite::params![format!("{}%", prefix)], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(dates)
}

#[tauri::command]
pub fn add_exercise_to_workout(
    date: &str,
    exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let workout_id = conn
        .query_row(
            "SELECT id FROM workouts WHERE date = ?1 AND workout_order = 1",
            rusqlite::params![date],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let workout_id = match workout_id {
        Some(workout_id) => workout_id,
        None => {
            conn.execute(
                "INSERT INTO workouts (date, workout_order) VALUES (?1, 1)",
                rusqlite::params![date],
            )
            .map_err(|e| e.to_string())?;
            conn.last_insert_rowid()
        }
    };

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

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn remove_exercise_from_workout(
    workout_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let workout_id = conn
        .query_row(
            "SELECT workout_id FROM workout_exercises WHERE id = ?1",
            rusqlite::params![workout_exercise_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE workout_exercises SET exercise_order = exercise_order - 1
            WHERE workout_id = (SELECT workout_id FROM workout_exercises WHERE id = ?1)
            AND exercise_order > (SELECT exercise_order FROM workout_exercises WHERE id = ?1)",
        rusqlite::params![workout_exercise_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM sets WHERE workout_exercise_id = ?1",
        rusqlite::params![workout_exercise_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM workout_exercises WHERE id = ?1",
        rusqlite::params![workout_exercise_id],
    )
    .map_err(|e| e.to_string())?;

    let exercise_count = conn
        .query_row(
            "SELECT COUNT(*) FROM workout_exercises WHERE workout_id = ?1",
            rusqlite::params![workout_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;

    if exercise_count == 0 {
        conn.execute(
            "DELETE FROM workouts WHERE id = ?1",
            rusqlite::params![workout_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
