use crate::models::{ExerciseWithSets, Set, TemplateWithExercises, WorkoutExerciseContext};
use crate::{database::recompute_pr_flags, models::NamedId};

#[tauri::command]
pub fn add_exercise_to_template(
    id: i64,
    exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    add_exercise_to_template_inner(&conn, id, exercise_id)
}

pub fn add_exercise_to_template_inner(
    conn: &rusqlite::Connection,
    template_id: i64,
    exercise_id: i64,
) -> Result<i64, String> {
    let next_exercise_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(exercise_order), 0) + 1 FROM template_exercises WHERE template_id = ?1",
            rusqlite::params![template_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO template_exercises (template_id, exercise_id, exercise_order) VALUES (?1, ?2, ?3)",
        rusqlite::params![template_id, exercise_id, next_exercise_order],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
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
pub fn create_template(
    name: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    create_template_inner(&conn, name)
}

pub fn create_template_inner(conn: &rusqlite::Connection, name: &str) -> Result<i64, String> {
    let mut stmt = conn
        .prepare("INSERT INTO templates (name) VALUES (?1) RETURNING id")
        .map_err(|e| e.to_string())?;
    let id = stmt
        .query_row(rusqlite::params![name], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn delete_template(
    template_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    delete_template_inner(&conn, template_id)
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
pub fn delete_template_set(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    delete_template_set_inner(&conn, id)
}

pub fn delete_template_set_inner(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM template_sets WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_sets_for_template_exercise(
    template_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Set>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_sets_for_template_exercise_inner(&conn, template_exercise_id)
}

pub fn get_sets_for_template_exercise_inner(
    conn: &rusqlite::Connection,
    template_exercise_id: i64,
) -> Result<Vec<Set>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.set_order, s.weight_kg, s.reps
             FROM template_sets s
             WHERE s.template_exercise_id = ?1
             ORDER BY s.set_order",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([template_exercise_id], |row| {
            Ok(Set {
                id: row.get(0)?,
                set_order: row.get(1)?,
                weight_kg: row.get(2)?,
                reps: row.get(3)?,
                notes: None,
                was_pr_at_time: false,
                is_current_pr: false,
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
pub fn get_template(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<TemplateWithExercises, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_template_inner(&conn, id)
}

pub fn get_template_inner(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<TemplateWithExercises, String> {
    let name = conn
        .query_row(
            "SELECT name FROM templates WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name, c.name, te.id, te.exercise_order, s.id, s.set_order, s.weight_kg, s.reps
            FROM templates t
            JOIN template_exercises te ON te.template_id = t.id
            JOIN exercises e ON te.exercise_id = e.id
            JOIN categories c ON e.category_id = c.id
            LEFT JOIN template_sets s ON s.template_exercise_id = te.id
            WHERE t.id = ?1
            ORDER BY te.exercise_order, s.set_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, i64>(0)?,         // exercise_id
                row.get::<_, String>(1)?,      // exercise_name
                row.get::<_, String>(2)?,      // category
                row.get::<_, i64>(3)?,         // template_exercise_id
                row.get::<_, i64>(4)?,         // exercise_order
                row.get::<_, Option<i64>>(5)?, // set_id
                row.get::<_, Option<i64>>(6)?, // set_order
                row.get::<_, Option<f64>>(7)?, // weight_kg
                row.get::<_, Option<i64>>(8)?, // reps
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: TemplateWithExercises = TemplateWithExercises {
        template: NamedId { id, name },
        exercises: Vec::new(),
    };
    for row in rows {
        let (
            exercise_id,
            exercise_name,
            category,
            template_exercise_id,
            exercise_order,
            set_id,
            set_order,
            weight_kg,
            reps,
        ) = row.map_err(|e| e.to_string())?;

        let exercise = match result.exercises.last_mut() {
            Some(e) if e.workout_exercise_id == template_exercise_id => e,
            _ => {
                result.exercises.push(ExerciseWithSets {
                    exercise: NamedId {
                        id: exercise_id,
                        name: exercise_name,
                    },
                    category,
                    workout_exercise_id: template_exercise_id,
                    exercise_order,
                    sets: Vec::new(),
                });
                result.exercises.last_mut().unwrap()
            }
        };

        if let Some(set_id) = set_id {
            let set = Set {
                id: set_id,
                set_order: set_order.unwrap_or_default(),
                weight_kg: weight_kg.unwrap_or_default(),
                reps: reps.unwrap_or_default(),
                notes: None,
                was_pr_at_time: false,
                is_current_pr: false,
            };
            exercise.sets.push(set);
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_template_exercise_context(
    template_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<WorkoutExerciseContext, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_template_exercise_context_inner(&conn, template_exercise_id)
}

pub fn get_template_exercise_context_inner(
    conn: &rusqlite::Connection,
    template_exercise_id: i64,
) -> Result<WorkoutExerciseContext, String> {
    let out = conn
        .query_row(
            "SELECT e.name
            FROM template_exercises te
            JOIN exercises e ON e.id = te.exercise_id
            WHERE te.id = ?1",
            rusqlite::params![template_exercise_id],
            |row| {
                Ok(WorkoutExerciseContext {
                    exercise_name: row.get::<_, String>(0)?,
                    date: "".to_string(),
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(out)
}

#[tauri::command]
pub fn list_templates(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<NamedId>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    list_templates_inner(&conn)
}

pub fn list_templates_inner(conn: &rusqlite::Connection) -> Result<Vec<NamedId>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM templates ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(NamedId {
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

pub fn merge_template_exercises_inner(
    conn: &rusqlite::Connection,
    template_exercise_ids: Vec<i64>,
) -> Result<(), String> {
    let mut exercise_orders = Vec::new();
    for te_id in &template_exercise_ids {
        let exercise_order = conn
            .query_row(
                "SELECT exercise_order FROM template_exercises WHERE id = ?1",
                rusqlite::params![te_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        exercise_orders.push((te_id, exercise_order));
    }

    exercise_orders.sort_by_key(|(_, order)| *order);

    let target_te_id = exercise_orders[0].0;
    let mut set_offset = 0;
    for (te_id, _) in exercise_orders {
        let set_count = conn
            .query_row(
                "SELECT COUNT(*) FROM template_sets WHERE template_exercise_id = ?1",
                rusqlite::params![te_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        set_offset += set_count;
        if te_id != target_te_id {
            conn.execute(
                "UPDATE template_sets SET template_exercise_id = ?1, set_order = set_order + ?2 WHERE template_exercise_id = ?3",
                rusqlite::params![target_te_id, set_offset, te_id],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "DELETE FROM template_exercises WHERE id = ?1",
                rusqlite::params![te_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn merge_template_exercises(
    template_exercise_ids: Vec<i64>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    merge_template_exercises_inner(&conn, template_exercise_ids)
}

pub fn remove_exercise_from_template_inner(
    conn: &rusqlite::Connection,
    template_exercise_id: i64,
) -> Result<(), String> {
    conn.execute(
        "UPDATE template_exercises SET exercise_order = exercise_order - 1
            WHERE template_id = (SELECT template_id FROM template_exercises WHERE id = ?1)
            AND exercise_order > (SELECT exercise_order FROM template_exercises WHERE id = ?1)",
        rusqlite::params![template_exercise_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM template_sets WHERE template_exercise_id = ?1",
        rusqlite::params![template_exercise_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM template_exercises WHERE id = ?1",
        rusqlite::params![template_exercise_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_exercise_from_template(
    template_exercise_id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    remove_exercise_from_template_inner(&conn, template_exercise_id)
}

pub fn reorder_template_exercises_inner(
    conn: &rusqlite::Connection,
    ordered_template_exercise_ids: Vec<i64>,
) -> Result<(), String> {
    for (i, template_exercise_id) in ordered_template_exercise_ids.iter().enumerate() {
        conn.execute(
            "UPDATE template_exercises SET exercise_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i64 + 1, template_exercise_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn reorder_template_exercises(
    ordered_template_exercise_ids: Vec<i64>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    reorder_template_exercises_inner(&conn, ordered_template_exercise_ids)
}

pub fn reorder_template_sets_inner(
    conn: &rusqlite::Connection,
    ordered_template_set_ids: Vec<i64>,
) -> Result<(), String> {
    for (i, set_id) in ordered_template_set_ids.iter().enumerate() {
        conn.execute(
            "UPDATE template_sets SET set_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i64 + 1, set_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn reorder_template_sets(
    ordered_template_set_ids: Vec<i64>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    reorder_template_sets_inner(&conn, ordered_template_set_ids)
}

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

pub fn upsert_template_set_inner(
    conn: &rusqlite::Connection,
    id: Option<i64>,
    template_exercise_id: i64,
    weight_kg: f64,
    reps: i64,
) -> Result<Set, String> {
    let template_id = conn
        .query_row(
            "SELECT template_id FROM template_exercises WHERE id = ?1",
            rusqlite::params![template_exercise_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;

    let set_id: i64;
    let set_order: i64;

    if let Some(existing_id) = id {
        // Update existing set
        set_id = existing_id;
        conn.execute(
            "UPDATE template_sets SET weight_kg = ?1, reps = ?2 WHERE id = ?4",
            rusqlite::params![weight_kg, reps, set_id],
        )
        .map_err(|e| e.to_string())?;
        set_order = conn
            .query_row(
                "SELECT set_order FROM template_sets WHERE id = ?1",
                rusqlite::params![set_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
    } else {
        // Insert new set
        set_order = conn
            .query_row(
                "SELECT COALESCE(MAX(set_order), 0) + 1 FROM template_sets WHERE template_exercise_id = ?1",
                rusqlite::params![template_exercise_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO template_sets (template_id, template_exercise_id, set_order, weight_kg, reps)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![template_id, template_exercise_id, set_order, weight_kg, reps],
        )
        .map_err(|e| e.to_string())?;
        set_id = conn.last_insert_rowid();
    }

    let was_pr_at_time = false;
    let is_current_pr = false;
    Ok(Set {
        id: set_id,
        set_order,
        weight_kg,
        reps,
        notes: None,
        was_pr_at_time,
        is_current_pr,
    })
}

#[tauri::command]
pub fn upsert_template_set(
    id: Option<i64>,
    template_exercise_id: i64,
    weight_kg: f64,
    reps: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Set, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    upsert_template_set_inner(&conn, id, template_exercise_id, weight_kg, reps)
}
