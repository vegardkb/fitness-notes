use crate::database::recompute_pr_flags;
use crate::models::Set;

#[tauri::command]
pub fn upsert_set(
    id: Option<i64>,
    workout_exercise_id: i64,
    weight_kg: f64,
    reps: i64,
    notes: Option<String>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Set, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let exercise_id = conn
        .query_row(
            "SELECT exercise_id FROM workout_exercises WHERE id = ?1",
            rusqlite::params![workout_exercise_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let set_id: i64;
    let set_order: i64;

    if let Some(existing_id) = id {
        // Update existing set
        set_id = existing_id;
        conn.execute(
            "UPDATE sets SET weight_kg = ?1, reps = ?2, notes = ?3 WHERE id = ?4",
            rusqlite::params![weight_kg, reps, notes, set_id],
        )
        .map_err(|e| e.to_string())?;
        set_order = conn
            .query_row(
                "SELECT set_order FROM sets WHERE id = ?1",
                rusqlite::params![set_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
    } else {
        // Insert new set
        set_order = conn
            .query_row(
                "SELECT COALESCE(MAX(set_order), 0) + 1 FROM sets WHERE workout_exercise_id = ?1",
                rusqlite::params![workout_exercise_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO sets (workout_exercise_id, exercise_id, set_order, weight_kg, reps, notes, was_pr_at_time, is_current_pr)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0)",
            rusqlite::params![workout_exercise_id, exercise_id, set_order, weight_kg, reps, notes],
        )
        .map_err(|e| e.to_string())?;
        set_id = conn.last_insert_rowid();
    }

    recompute_pr_flags(&conn, exercise_id)?;

    let (was_pr_at_time, is_current_pr) = conn
        .query_row(
            "SELECT was_pr_at_time, is_current_pr FROM sets WHERE id = ?1",
            rusqlite::params![set_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(Set {
        id: set_id,
        set_order,
        weight_kg,
        reps,
        notes,
        was_pr_at_time,
        is_current_pr,
    })
}

#[tauri::command]
pub fn delete_set(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let exercise_id = conn
        .query_row(
            "SELECT exercise_id FROM sets WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok(row.get::<_, i64>(0)?),
        )
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM sets WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    recompute_pr_flags(&conn, exercise_id)?;

    Ok(())
}

#[tauri::command]
pub fn reorder_sets(
    workout_exercise_id: i64,
    ordered_set_ids: Vec<i64>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    for (i, set_id) in ordered_set_ids.iter().enumerate() {
        conn.execute(
            "UPDATE sets SET set_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i64 + 1, set_id],
        )
        .map_err(|e| e.to_string())?;
    }
    let exercise_id = conn
        .query_row(
            "SELECT exercise_id FROM workout_exercises WHERE id = ?1",
            rusqlite::params![workout_exercise_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    recompute_pr_flags(&conn, exercise_id)?;
    Ok(())
}

#[tauri::command]
pub fn reorder_exercises(
    ordered_workout_exercise_ids: Vec<i64>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    for (i, workout_exercise_id) in ordered_workout_exercise_ids.iter().enumerate() {
        conn.execute(
            "UPDATE workout_exercises SET exercise_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i64 + 1, workout_exercise_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
