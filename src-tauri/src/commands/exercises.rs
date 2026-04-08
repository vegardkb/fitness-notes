use crate::database::recompute_pr_flags;
use crate::models::{
    Category, DatedValue, DayWorkout, Exercise, ExerciseWithSets, RepMax, Set, SetMinimal,
};

#[tauri::command]
pub fn list_exercise_categories(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Category>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT name, id FROM categories")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(Category {
                name: row.get(0)?,
                id: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<Category> = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }

    Ok(result)
}

#[tauri::command]
pub fn list_exercises_in_category(
    category_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Exercise>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name FROM exercises e
             WHERE e.category_id = ?1
             ORDER BY e.name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![category_id], |row| {
            Ok(Exercise {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn create_exercise(
    name: &str,
    category_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO exercises (name, category_id) VALUES (?1, ?2)",
        rusqlite::params![name, category_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_exercise(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let set_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sets WHERE exercise_id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if set_count > 0 {
        return Err("Cannot delete exercise with existing sets".to_string());
    }
    conn.execute("DELETE FROM exercises WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn rename_exercise(
    id: i64,
    name: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let exercise_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM exercises WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exercise_count == 0 {
        return Err("Exercise not found".to_string());
    }

    let target_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM exercises WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if target_count > 0 {
        return Err("An exercise named X already exists — merge Y into X?".to_string());
    }

    conn.execute(
        "UPDATE exercises SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_category(
    name: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO categories (name) VALUES (?1)",
        rusqlite::params![name],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn rename_category(
    id: i64,
    name: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let category_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if category_count == 0 {
        return Err("Category not found".to_string());
    }
    conn.execute(
        "UPDATE categories SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_category(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let set_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sets s
             JOIN exercises e ON s.exercise_id = e.id
             WHERE e.category_id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if set_count > 0 {
        return Err("Cannot delete category with existing sets".to_string());
    }
    let exercise_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM exercises WHERE category_id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exercise_count > 0 {
        return Err("Cannot delete category with existing exercises".to_string());
    }
    conn.execute(
        "DELETE FROM categories WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn merge_exercise_into_existing(
    from_id: i64,
    to_name: String,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let to_id: i64 = tx
        .query_row(
            "SELECT id FROM exercises WHERE name = ?1",
            rusqlite::params![to_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM workout_exercises
         WHERE exercise_id = ?1
           AND workout_id IN (
             SELECT workout_id FROM workout_exercises WHERE exercise_id = ?2
           )",
        rusqlite::params![from_id, to_id],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE workout_exercises SET exercise_id = ?1 WHERE exercise_id = ?2",
        rusqlite::params![to_id, from_id],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE sets SET exercise_id = ?1 WHERE exercise_id = ?2",
        rusqlite::params![to_id, from_id],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM exercises WHERE id = ?1",
        rusqlite::params![from_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    recompute_pr_flags(&conn, to_id).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_exercise(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Exercise, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name FROM exercises WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Exercise {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_exercise_history(
    exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DayWorkout>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT w.date, e.id, e.name, c.name, we.exercise_order,
                    s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN categories c ON e.category_id = c.id
             JOIN sets s ON s.workout_id = w.id AND s.exercise_id = e.id
             WHERE e.id = ?1
             ORDER BY w.date DESC, we.exercise_order, s.set_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![exercise_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, f64>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, bool>(10)?,
                row.get::<_, bool>(11)?,
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
                exercise_order,
                sets: vec![set],
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_exercise_graph_data(
    exercise_id: i64,
    from_date: &str,
    to_date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DatedValue>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT w.date, s.weight_kg, s.reps
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN sets s ON s.workout_id = w.id AND s.exercise_id = e.id
             WHERE e.id = ?1 AND w.date BETWEEN ?2 AND ?3
             ORDER BY w.date DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![exercise_id, from_date, to_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut day_max: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for row in rows {
        let (date, weight, reps) = row.map_err(|e| e.to_string())?;
        let rm = estimate_1rm(weight, reps);
        let entry = day_max.entry(date).or_insert(0.0);
        if rm > *entry {
            *entry = rm;
        }
    }
    let mut result: Vec<DatedValue> = day_max
        .into_iter()
        .map(|(date, value)| DatedValue { date, value })
        .collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(result)
}

#[tauri::command]
pub fn get_rep_maxes(
    exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<RepMax>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT w.date, s.weight_kg, s.reps
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN sets s ON s.workout_id = w.id AND s.exercise_id = e.id
             WHERE e.id = ?1 AND s.is_current_pr = 1
             ORDER BY s.weight_kg DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![exercise_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<RepMax> = Vec::new();
    for row in rows {
        let (date, weight_kg, reps) = row.map_err(|e| e.to_string())?;

        result.push(RepMax {
            date,
            reps,
            weight_kg,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn get_last_set(
    exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<SetMinimal, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT s.weight_kg, s.reps FROM sets s
             JOIN workouts w ON w.id = s.workout_id
             WHERE s.exercise_id = ?1
             ORDER BY w.date DESC, s.set_order DESC
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let row = stmt
        .query_row(rusqlite::params![exercise_id], |row| {
            Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let (weight, reps) = row;

    Ok(SetMinimal { weight, reps })
}

fn estimate_1rm(weight: f64, reps: i64) -> f64 {
    // Use the Brzycki formula to estimate 1RM for reps <= 10
    // Epley for reps > 10
    if reps <= 0 {
        0.0
    } else if reps <= 10 {
        weight * (36.0 / (37.0 - reps as f64))
    } else {
        weight * (1.0 + (reps as f64 / 30.0))
    }
}
