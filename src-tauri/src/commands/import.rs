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

    let mut parsed: Vec<ExerciseRow> = Vec::new();
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
        parsed.push(ExerciseRow {
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
pub fn parse_body_measurements_csv(
    csv: String,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<ParseBodyResult, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut known: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut stmt = conn
        .prepare("SELECT id, name FROM body_metrics")
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

    let mut parsed: Vec<BodyRow> = Vec::new();
    let mut unknowns_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unknown_metrics: Vec<String> = Vec::new();

    for (i, line) in csv.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 5 {
            continue;
        }
        let date = cols[0].trim().to_string();
        let measurement = cols[2].trim().to_string();
        let value_raw: f64 = match cols[3].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let unit = cols[4].trim().to_string();

        let value = if unit == "lbs" || unit == "lb" {
            value_raw * 0.453592
        } else {
            value_raw
        };
        if date.is_empty() || measurement.is_empty() {
            continue;
        }

        let metric_id = known.get(&measurement).copied();
        if metric_id.is_none() && unknowns_seen.insert(measurement.clone()) {
            unknown_metrics.push(measurement.clone());
        }
        parsed.push(BodyRow {
            date,
            measurement,
            value,
            unit,
            metric_id,
        });
    }

    Ok(ParseBodyResult {
        rows: parsed,
        unknown_metrics,
    })
}

#[tauri::command]
pub fn import_fitnotes_rows(
    rows: Vec<ExerciseRow>,
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

#[tauri::command]
pub fn import_body_measurement_rows(
    rows: Vec<BodyRow>,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<BodyImportResult, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut name_to_id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut affected_dates: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut measurements_imported = 0;

    for row in rows {
        // Resolve metric_id: use existing if available, insert if not
        let metric_id = if let Some(id) = row.metric_id {
            id
        } else if let Some(&id) = name_to_id.get(&row.measurement) {
            id
        } else {
            let unit = match row.unit.as_str() {
                "kg" | "kgs" | "lbs" | "lb" => "kg",
                "cm" => "cm",
                "%" => "%",
                _ => continue,
            };
            conn.execute(
                "INSERT OR IGNORE INTO body_metrics (name, unit) VALUES (?1, ?2)",
                rusqlite::params![row.measurement, unit],
            )
            .map_err(|e| e.to_string())?;
            let m_id: i64 = conn
                .query_row(
                    "SELECT id FROM body_metrics WHERE name = ?1",
                    rusqlite::params![row.measurement],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            name_to_id.insert(row.measurement.clone(), m_id);
            m_id
        };

        // Check for existing measurement for this date and metric
        let num_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM body_measurements WHERE date = ?1 AND measure_id = ?2",
                rusqlite::params![row.date, metric_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if num_rows > 0 {
            continue;
        }

        // Check if metric is derived
        let derived: bool = conn
            .query_row(
                "SELECT is_derived FROM body_metrics WHERE id = ?1",
                rusqlite::params![metric_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if derived {
            continue;
        }

        // Insert new measurement
        conn.execute(
            "INSERT INTO body_measurements (date, value, measure_id)
            VALUES (?1, ?2, ?3)",
            rusqlite::params![row.date, row.value, metric_id],
        )
        .map_err(|e| e.to_string())?;
        measurements_imported += 1;
        affected_dates.insert(row.date);
    }
    let days_touched = affected_dates.len() as i64;

    Ok(BodyImportResult {
        measurements_imported,
        days_touched,
    })
}

#[derive(Serialize, Deserialize)]
pub struct ExerciseRow {
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
    rows: Vec<ExerciseRow>,
    unknown_exercises: Vec<UnknownExercise>,
}

#[derive(Serialize)]
pub struct ImportResult {
    sets_imported: i64,
    workouts_touched: i64,
}

#[derive(Serialize, Deserialize)]
pub struct BodyRow {
    date: String,
    measurement: String,
    value: f64,
    unit: String,
    metric_id: Option<i64>,
}

#[derive(Serialize)]
pub struct ParseBodyResult {
    rows: Vec<BodyRow>,
    unknown_metrics: Vec<String>,
}

#[derive(Serialize)]
pub struct BodyImportResult {
    measurements_imported: i64,
    days_touched: i64,
}
