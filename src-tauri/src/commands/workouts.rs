use crate::models::{DayWorkout, ExerciseWithSets, Set, WorkoutExerciseContext};
use rusqlite::OptionalExtension;

pub fn get_workout_for_date_inner(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Vec<ExerciseWithSets>, String> {
    let mut stmt = conn
        .prepare(
            "select e.id, e.name, c.name, we.id, we.exercise_order
        FROM workouts w
        JOIN workout_exercises we ON we.workout_id = w.id
        JOIN exercises e ON we.exercise_id = e.id
        JOIN categories c ON e.category_id = c.id
        WHERE w.date = ?1
        ORDER BY we.exercise_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, i64>(0)?,    // exercise_id
                row.get::<_, String>(1)?, // exercise_name
                row.get::<_, String>(2)?, // category
                row.get::<_, i64>(3)?,    // workout_exercise_id
                row.get::<_, i64>(4)?,    // exercise_order
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<ExerciseWithSets> = Vec::new();
    for row in rows {
        let (exercise_id, exercise_name, category, workout_exercise_id, exercise_order) =
            row.map_err(|e| e.to_string())?;

        let mut ex_with_sets = ExerciseWithSets {
            exercise_id,
            exercise_name,
            category,
            workout_exercise_id,
            exercise_order,
            sets: Vec::new(),
        };

        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
                 FROM sets s
                 WHERE s.workout_exercise_id = ?1
                 ORDER BY s.set_order",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(rusqlite::params![workout_exercise_id], |row| {
                Ok(Set {
                    id: row.get::<_, i64>(0)?,               // set id
                    set_order: row.get::<_, i64>(1)?,        // set_order
                    weight_kg: row.get::<_, f64>(2)?,        // weight_kg
                    reps: row.get::<_, i64>(3)?,             // reps
                    notes: row.get::<_, Option<String>>(4)?, // notes
                    was_pr_at_time: row.get::<_, bool>(5)?,  // was_pr_at_time
                    is_current_pr: row.get::<_, bool>(6)?,   // is_current_pr
                })
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            ex_with_sets.sets.push(row.map_err(|e| e.to_string())?);
        }
        result.push(ex_with_sets);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_sets_for_workout_exercise(
    workout_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Set>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_sets_for_workout_exercise_inner(&conn, workout_exercise_id)
}

pub fn get_sets_for_workout_exercise_inner(
    conn: &rusqlite::Connection,
    workout_exercise_id: i64,
) -> Result<Vec<Set>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM sets s
             WHERE s.workout_exercise_id = ?1
             ORDER BY s.set_order",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([workout_exercise_id], |row| {
            Ok(Set {
                id: row.get(0)?,
                set_order: row.get(1)?,
                weight_kg: row.get(2)?,
                reps: row.get(3)?,
                notes: row.get(4)?,
                was_pr_at_time: row.get(5)?,
                is_current_pr: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut result: Vec<Set> = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
pub fn get_workout_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<ExerciseWithSets>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_workout_for_date_inner(&conn, date)
}

pub fn get_workouts_for_range_inner(
    conn: &rusqlite::Connection,
    from_date: &str,
    to_date: &str,
) -> Result<Vec<DayWorkout>, String> {
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
pub fn get_workouts_for_range(
    from_date: &str,
    to_date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DayWorkout>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_workouts_for_range_inner(&conn, from_date, to_date)
}

pub fn get_active_dates_inner(
    conn: &rusqlite::Connection,
    year: i32,
    month: u32,
) -> Result<Vec<String>, String> {
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
pub fn get_active_dates(
    year: i32,
    month: u32,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<String>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_active_dates_inner(&conn, year, month)
}

#[tauri::command]
pub fn add_exercise_to_workout(
    date: &str,
    exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    add_exercise_to_workout_inner(&conn, date, exercise_id)
}

pub fn add_exercise_to_workout_inner(
    conn: &rusqlite::Connection,
    date: &str,
    exercise_id: i64,
) -> Result<i64, String> {
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
        "INSERT INTO workout_exercises (workout_id, exercise_id, exercise_order) VALUES (?1, ?2, ?3)",
        rusqlite::params![workout_id, exercise_id, next_exercise_order],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

pub fn remove_exercise_from_workout_inner(
    conn: &rusqlite::Connection,
    workout_exercise_id: i64,
) -> Result<(), String> {
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

#[tauri::command]
pub fn remove_exercise_from_workout(
    workout_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    remove_exercise_from_workout_inner(&conn, workout_exercise_id)
}

pub fn get_workout_exercise_context_inner(
    conn: &rusqlite::Connection,
    workout_exercise_id: i64,
) -> Result<WorkoutExerciseContext, String> {
    let out = conn
        .query_row(
            "SELECT e.name, w.date
            FROM workout_exercises we
            JOIN exercises e ON e.id = we.exercise_id
            JOIN workouts w ON we.workout_id = w.id
            WHERE we.id = ?1",
            rusqlite::params![workout_exercise_id],
            |row| {
                Ok(WorkoutExerciseContext {
                    exercise_name: row.get::<_, String>(0)?,
                    date: row.get::<_, String>(1)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(out)
}

#[tauri::command]
pub fn get_workout_exercise_context(
    workout_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<WorkoutExerciseContext, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_workout_exercise_context_inner(&conn, workout_exercise_id)
}

pub fn merge_workout_exercises_inner(
    conn: &rusqlite::Connection,
    workout_exercise_ids: Vec<i64>,
) -> Result<(), String> {
    // Steps:
    // 1. Sort workout_exercise by exercise_order
    // 2. update sets (we_id -> target_we_id, set_order -> set_order + offset)
    // 3. delete we_id != target_we_id

    let mut exercise_orders = Vec::new();
    for we_id in &workout_exercise_ids {
        let exercise_order = conn
            .query_row(
                "SELECT exercise_order FROM workout_exercises WHERE id = ?1",
                rusqlite::params![we_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        exercise_orders.push((we_id, exercise_order));
    }

    exercise_orders.sort_by_key(|(_, order)| *order);

    let target_we_id = exercise_orders[0].0;
    let mut set_offset = 0;
    for (we_id, _) in exercise_orders {
        let set_count = conn
            .query_row(
                "SELECT COUNT(*) FROM sets WHERE workout_exercise_id = ?1",
                rusqlite::params![we_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        set_offset += set_count;
        if we_id != target_we_id {
            conn.execute(
                "UPDATE sets SET workout_exercise_id = ?1, set_order = set_order + ?2 WHERE workout_exercise_id = ?3",
                rusqlite::params![target_we_id, set_offset, we_id],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "DELETE FROM workout_exercises WHERE id = ?1",
                rusqlite::params![we_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn merge_workout_exercises(
    workout_exercise_ids: Vec<i64>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    merge_workout_exercises_inner(&conn, workout_exercise_ids)
}
