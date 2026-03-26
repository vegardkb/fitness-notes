use crate::database;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub fn parse_fitnotes_csv(
    csv_text: String,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<ParseResult, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut known: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut stmt = conn
        .prepare("SELECT id, name FROM exercises")
        .map_err(|e| e.to_string())?;
    let rows_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    for r in rows_iter {
        let (id, name) = r.map_err(|e| e.to_string())?;
        known.insert(name, id);
    }

    let mut parsed: Vec<ParsedRow> = Vec::new();
    let mut unknowns_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unknown_exercises: Vec<UnknownExercise> = Vec::new();

    for (i, line) in csv_text.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 6 {
            continue;
        }
        let date = cols[0].trim().to_string();
        let exercise_name = cols[1].trim().to_string();
        let category_name = cols[2].trim().to_string();
        let weight_raw: f64 = cols[3].trim().parse().unwrap_or(0.0);
        let weight_unit = cols[4].trim().to_lowercase();
        let weight_kg = if weight_unit == "lbs" || weight_unit == "lb" {
            weight_raw * 0.453592
        } else {
            weight_raw
        };
        let reps: i64 = cols[5].trim().parse().unwrap_or(0);
        if reps == 0 {
            continue; // skip cardio/empty rows
        }
        if date.is_empty() || exercise_name.is_empty() {
            continue;
        }

        let exercise_id = known.get(&exercise_name).copied();
        if exercise_id.is_none() && unknowns_seen.insert(exercise_name.clone()) {
            unknown_exercises.push(UnknownExercise {
                csv_name: exercise_name.clone(),
                csv_category: category_name.clone(),
            });
        }
        parsed.push(ParsedRow {
            date,
            exercise_name,
            category_name,
            weight_kg,
            reps,
            exercise_id,
        });
    }

    Ok(ParseResult {
        rows: parsed,
        unknown_exercises,
    })
}

#[tauri::command]
pub fn import_fitnotes_rows(
    rows: Vec<ResolvedRow>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<ImportResult, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut name_to_id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut date_to_workout: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    let mut set_order_counter: std::collections::HashMap<(i64, i64), i64> =
        std::collections::HashMap::new();
    let mut affected_exercises: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut workouts_touched: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut sets_imported: i64 = 0;

    for row in &rows {
        // 1. Resolve exercise_id
        let exercise_id = if let Some(id) = row.exercise_id {
            id
        } else if let Some(&id) = name_to_id.get(&row.exercise_name) {
            id
        } else {
            conn.execute(
                "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
                rusqlite::params![row.category_name],
            )
            .map_err(|e| e.to_string())?;
            let cat_id: i64 = conn
                .query_row(
                    "SELECT id FROM categories WHERE name = ?1",
                    rusqlite::params![row.category_name],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT OR IGNORE INTO exercises (name, category_id) VALUES (?1, ?2)",
                rusqlite::params![row.exercise_name, cat_id],
            )
            .map_err(|e| e.to_string())?;
            let ex_id: i64 = conn
                .query_row(
                    "SELECT id FROM exercises WHERE name = ?1",
                    rusqlite::params![row.exercise_name],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            name_to_id.insert(row.exercise_name.clone(), ex_id);
            ex_id
        };

        // 2. Find or create workout
        let workout_id = if let Some(&wid) = date_to_workout.get(&row.date) {
            wid
        } else {
            conn.execute(
                "INSERT OR IGNORE INTO workouts (date) VALUES (?1)",
                rusqlite::params![row.date],
            )
            .map_err(|e| e.to_string())?;
            let wid: i64 = conn
                .query_row(
                    "SELECT id FROM workouts WHERE date = ?1",
                    rusqlite::params![row.date],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            date_to_workout.insert(row.date.clone(), wid);
            workouts_touched.insert(wid);
            wid
        };

        // 3. Find or create workout_exercises entry
        let next_ex_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(exercise_order), 0) + 1 FROM workout_exercises WHERE workout_id = ?1",
                rusqlite::params![workout_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO workout_exercises (workout_id, exercise_id, exercise_order) VALUES (?1, ?2, ?3)",
            rusqlite::params![workout_id, exercise_id, next_ex_order],
        )
        .map_err(|e| e.to_string())?;

        // 4. Insert set
        let key = (workout_id, exercise_id);
        let set_order = set_order_counter.entry(key).or_insert_with(|| {
            conn.query_row(
                "SELECT COALESCE(MAX(set_order), 0) + 1 FROM sets WHERE workout_id = ?1 AND exercise_id = ?2",
                rusqlite::params![workout_id, exercise_id],
                |r| r.get(0),
            )
            .unwrap_or(1)
        });
        conn.execute(
            "INSERT INTO sets (workout_id, exercise_id, set_order, weight_kg, reps, notes, was_pr_at_time, is_current_pr)
            VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, 0)",
            rusqlite::params![workout_id, exercise_id, *set_order, row.weight_kg, row.reps],
        )
        .map_err(|e| e.to_string())?;
        *set_order += 1;
        affected_exercises.insert(exercise_id);
        sets_imported += 1;
    }

    for ex_id in &affected_exercises {
        database::recompute_pr_flags(&conn, *ex_id)?;
    }

    Ok(ImportResult {
        sets_imported,
        workouts_touched: workouts_touched.len() as i64,
    })
}

#[derive(Serialize)]
struct ParsedRow {
    date: String,
    exercise_name: String,
    category_name: String,
    weight_kg: f64,
    reps: i64,
    exercise_id: Option<i64>,
}

#[derive(Serialize)]
struct UnknownExercise {
    csv_name: String,
    csv_category: String,
}

#[derive(Serialize)]
pub struct ParseResult {
    rows: Vec<ParsedRow>,
    unknown_exercises: Vec<UnknownExercise>,
}

#[derive(Deserialize)]
pub struct ResolvedRow {
    date: String,
    exercise_id: Option<i64>,
    exercise_name: String,
    category_name: String,
    weight_kg: f64,
    reps: i64,
}

#[derive(Serialize)]
pub struct ImportResult {
    sets_imported: i64,
    workouts_touched: i64,
}
