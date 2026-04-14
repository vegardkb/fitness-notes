use crate::{database::recompute_pr_flags, models::Template};

pub fn rename_template_inner(
    conn: &rusqlite::Connection,
    id: i64,
    name: String,
) -> Result<(), String> {
    conn.execute("UPDATE templates SET name = ? WHERE id = ?", (name, id))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn rename_template(
    id: i64,
    name: String,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    rename_template_inner(&conn, id, name)
}

pub fn save_workout_as_template_inner(
    conn: &mut rusqlite::Connection,
    workout_id: i64,
    name: String,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO templates (name) VALUES (?1)", (name,))
        .map_err(|e| e.to_string())?;

    let template_id = tx.last_insert_rowid();

    let workout_exercises: Result<Vec<(i64, i64, i64)>, String> = {
        let mut stmt = tx
            .prepare(
                "SELECT we.id, we.exercise_id, we.exercise_order
                 FROM workout_exercises we
                 WHERE we.workout_id = ?1
                 ORDER BY we.exercise_order",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([workout_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|r| r.map_err(|e| e.to_string())).collect()
    };

    for (workout_exercise_id, exercise_id, exercise_order) in workout_exercises? {
        tx.execute(
            "INSERT INTO template_exercises (template_id, exercise_id, exercise_order) VALUES (?, ?, ?)",
            (template_id, exercise_id, exercise_order),
        ).map_err(|e| e.to_string())?;

        let template_exercise_id = tx.last_insert_rowid();
        let mut stmt = tx
            .prepare(
                "SELECT set_order, weight_kg, reps
                FROM sets
                WHERE workout_exercise_id = ?",
            )
            .map_err(|e| e.to_string())?;

        let sets = stmt
            .query_map(rusqlite::params![workout_exercise_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        for set in sets {
            let (set_order, weight_kg, reps) = set.map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO template_sets (template_id, template_exercise_id, set_order, weight_kg, reps) VALUES (?, ?, ?, ?, ?)",
                (template_id, template_exercise_id, set_order, weight_kg, reps),
            ).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_workout_as_template(
    workout_id: i64,
    name: String,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    save_workout_as_template_inner(&mut conn, workout_id, name)
}

pub fn apply_template_inner(
    conn: &mut rusqlite::Connection,
    template_id: i64,
    date: String,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let workout_count = tx
        .query_row(
            "SELECT COUNT(*) FROM workouts WHERE date = ?1",
            rusqlite::params![date],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;
    if workout_count > 0 {
        return Err("A workout already exists for this date".to_string());
    }

    let name = tx
        .query_row(
            "SELECT name FROM templates WHERE id = ?1",
            rusqlite::params![template_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO workouts (date, workout_order, name) VALUES (?1, 1, ?2)",
        rusqlite::params![date, name],
    )
    .map_err(|e| e.to_string())?;

    let workout_id = tx.last_insert_rowid();

    let rows: Result<Vec<_>, _> = {
        let mut stmt = tx
            .prepare(
                "SELECT te.id, te.exercise_id, te.exercise_order
                     FROM template_exercises te
                     WHERE te.template_id = ?1
                     ORDER BY te.exercise_order",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([template_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.collect()
    };

    let rows = rows.map_err(|e| e.to_string())?;

    for (template_exercise_id, exercise_id, exercise_order) in rows.iter() {
        tx.execute(
            "INSERT INTO workout_exercises (workout_id, exercise_id, exercise_order)
                VALUES (?1, ?2, ?3)",
            rusqlite::params![workout_id, exercise_id, exercise_order],
        )
        .map_err(|e| e.to_string())?;
        let workout_exercise_id = tx.last_insert_rowid();

        let mut stmt = tx
            .prepare(
                "SELECT set_order, weight_kg, reps
                    FROM template_sets
                    WHERE template_exercise_id = ?",
            )
            .map_err(|e| e.to_string())?;

        let sets = stmt
            .query_map(rusqlite::params![template_exercise_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        for set in sets {
            let (set_order, weight_kg, reps) = set.map_err(|e| e.to_string())?;
            tx.execute(
                    "INSERT INTO sets (workout_exercise_id, exercise_id, set_order, weight_kg, reps, notes, was_pr_at_time, is_current_pr)
                        VALUES (?1, ?2, ?3, ?4, ?5, null, 0, 0)",
                    rusqlite::params![workout_exercise_id, exercise_id, set_order, weight_kg, reps],
                ).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    for (_, exercise_id, _) in rows {
        recompute_pr_flags(conn, exercise_id)?;
    }

    Ok(())
}

#[tauri::command]
pub fn apply_template(
    template_id: i64,
    date: String,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    apply_template_inner(&mut conn, template_id, date)
}

pub fn delete_template_inner(conn: &rusqlite::Connection, template_id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM template_sets
            WHERE template_id = ?1",
        rusqlite::params![template_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM template_exercises
            WHERE template_id = ?1",
        rusqlite::params![template_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM templates
            WHERE id = ?1",
        rusqlite::params![template_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_template(
    template_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    delete_template_inner(&conn, template_id)
}

pub fn list_templates_inner(conn: &rusqlite::Connection) -> Result<Vec<Template>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM templates ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Template {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut templates = Vec::new();
    for row in rows {
        templates.push(row.map_err(|e| e.to_string())?);
    }
    Ok(templates)
}

#[tauri::command]
pub fn list_templates(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Template>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    list_templates_inner(&conn)
}
