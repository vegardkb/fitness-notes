use crate::models::{DayMeasurement, Measurement, Metric, Sex};
use rusqlite::OptionalExtension;
use std::collections::HashMap;

use crate::{database::get_settings, models::Settings};

/*
 Commands:
- get_last_measurements_for_date(date)
 -> Vec<Measurement>
- upsert_measurement(Optional(id), metric, value, date)
 -> ()
- delete_measurement(measurement_id)
 -> ()
*/

#[tauri::command]
pub fn upsert_body_measurement(
    id: Option<i64>,
    date: &str,
    measure_id: i64,
    value: f64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    println!("In upsert_body_measurement");

    // This block will be repeated 3 times, may turn into a function
    let row_id = match upsert_body_measurement_clean(&conn, id, date, measure_id, value) {
        Ok(row_id) => row_id,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    // Recompute derived metrics after upsert
    // Recompute body fat percentage if measure_id corresponds to (waist_cm, neck_cm) if male or (waist_cm, hip_cm, neck_cm) if female
    // Recompute ffmi if weight or body fat percentage is upserted
    let measure = match conn
        .query_row(
            "SELECT name FROM body_metrics WHERE id = ?1",
            rusqlite::params![measure_id],
            |row| Ok(row.get::<_, String>(0)?),
        )
        .map_err(|e| e.to_string())
    {
        Ok(measure) => measure,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    // query settings to get height, sex and body fat percentage preference
    let settings = match get_settings(&conn) {
        Ok(settings) => settings,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let mut should_recompute_bf = settings.estimate_body_fat;
    should_recompute_bf &= match settings.sex {
        Sex::Male => measure == "Waist" || measure == "Neck",
        Sex::Female => measure == "Waist" || measure == "Neck" || measure == "Hip",
    };

    if should_recompute_bf {
        recompute_body_fat(&conn, &settings, date)?;
    }

    let should_recompute_ffmi = should_recompute_bf || measure == "Body Fat" || measure == "Weight";

    if should_recompute_ffmi {
        recompute_ffmi(&conn, &settings, date)?;
    }

    Ok(row_id)
}

#[tauri::command]
pub fn delete_body_measurement(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM body_measurements WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_last_measurements_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Measurement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = match conn
        .prepare(
            "SELECT bm.name, b.value, bm.unit, bm.id, b.id, b.date
             FROM body_measurements b
             JOIN body_metrics bm ON b.measure_id = bm.id
             WHERE b.date = (
                SELECT MAX(b2.date)
                FROM body_measurements b2
                WHERE b2.measure_id = b.measure_id
                    AND b2.date <= ?1
            )",
        )
        .map_err(|e| e.to_string())
    {
        Ok(stmt) => stmt,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let rows = match stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, String>(0)?, // metric name
                row.get::<_, f64>(1)?,    // value
                row.get::<_, String>(2)?, // unit of measurement
                row.get::<_, i64>(3)?,    // metric id
                row.get::<_, i64>(4)?,    // body measurement id
                row.get::<_, String>(5)?, // date
            ))
        })
        .map_err(|e| e.to_string())
    {
        Ok(rows) => rows,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let mut result: Vec<Measurement> = Vec::new();
    for row in rows {
        let (name, value, unit, metric_id, id, measure_date) = row.map_err(|e| e.to_string())?;

        result.push(Measurement {
            metric: Metric {
                name,
                unit,
                id: metric_id,
            },
            value,
            date: measure_date,
            id,
        });
    }

    dbg!(&result);

    Ok(result)
}

#[tauri::command]
pub fn get_measurement_history(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DayMeasurement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = match conn
        .prepare(
            "SELECT bm.name, b.value, bm.unit, bm.id, b.id, b.date
             FROM body_measurements b
             JOIN body_metrics bm ON b.measure_id = bm.id
             ORDER BY b.date DESC, bm.name",
        )
        .map_err(|e| e.to_string())
    {
        Ok(stmt) => stmt,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let rows = match stmt
        .query_map(rusqlite::params![], |row| {
            Ok((
                row.get::<_, String>(0)?, // metric name
                row.get::<_, f64>(1)?,    // value
                row.get::<_, String>(2)?, // unit of measurement
                row.get::<_, i64>(3)?,    // metric id
                row.get::<_, i64>(4)?,    // body measurement id
                row.get::<_, String>(5)?, // date
            ))
        })
        .map_err(|e| e.to_string())
    {
        Ok(rows) => rows,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let mut result: Vec<DayMeasurement> = Vec::new();
    for row in rows {
        let (metric_name, value, unit, metric_id, id, date) = row.map_err(|e| e.to_string())?;
        let metric = Metric {
            name: metric_name,
            unit,
            id: metric_id,
        };
        let measurement = Measurement {
            metric,
            value,
            date: date.clone(),
            id,
        };

        let day = match result.last_mut() {
            Some(d) if d.date == date => d,
            _ => {
                result.push(DayMeasurement {
                    date: date.clone(),
                    measurements: Vec::new(),
                });
                result.last_mut().unwrap()
            }
        };
        day.measurements.push(measurement);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_measurements_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Measurement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = match conn
        .prepare(
            "SELECT bm.name, b.value, bm.unit, bm.id, b.id
             FROM body_measurements b
             JOIN body_metrics bm ON b.measure_id = bm.id
             WHERE b.date = ?1",
        )
        .map_err(|e| e.to_string())
    {
        Ok(stmt) => stmt,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let rows = match stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, String>(0)?, // metric name
                row.get::<_, f64>(1)?,    // value
                row.get::<_, String>(2)?, // unit of measurement
                row.get::<_, i64>(3)?,    // metric id
                row.get::<_, i64>(4)?,    // measurement id
            ))
        })
        .map_err(|e| e.to_string())
    {
        Ok(rows) => rows,
        Err(e) => {
            dbg!(&e);
            return Err(e);
        }
    };

    let mut result: Vec<Measurement> = Vec::new();
    for row in rows {
        let (name, value, unit, metric_id, id) = row.map_err(|e| e.to_string())?;

        result.push(Measurement {
            metric: Metric {
                name,
                unit,
                id: metric_id,
            },
            value,
            date: date.to_string(),
            id,
        });
    }

    dbg!(&result);

    Ok(result)
}

#[tauri::command]
pub fn list_metrics(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Metric>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT bm.name, bm.unit, bm.id FROM body_metrics bm
             ORDER BY bm.name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(Metric {
                name: row.get::<_, String>(0)?,
                unit: row.get::<_, String>(1)?,
                id: row.get::<_, i64>(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let out = rows.map(|r| r.map_err(|e| e.to_string())).collect();
    dbg!(&out);

    out
}

fn recompute_body_fat(
    conn: &rusqlite::Connection,
    settings: &Settings,
    date: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT m.name, b.value FROM body_measurements b
             JOIN body_metrics m ON b.measure_id = m.id
             WHERE b.date = ?1",
        )
        .map_err(|e| e.to_string())?;

    let measures: Vec<(String, f64)> = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut can_recompute_body_fat = true;
    can_recompute_body_fat &= measures.iter().any(|(m, _)| m == "Waist");
    can_recompute_body_fat &= measures.iter().any(|(m, _)| m == "Neck");
    can_recompute_body_fat &= match settings.sex {
        Sex::Male => true,
        Sex::Female => measures.iter().any(|(m, _)| m == "Hip"),
    };

    let measures: HashMap<String, f64> = measures.into_iter().collect();
    if !can_recompute_body_fat {
        println!("Cannot recompute body fat: missing required measurements or permissions");
        return Ok(());
    }
    // unwrap should be safe because we have checked that the fields exist, but feels a bit clunky
    let bf = match settings.sex {
        Sex::Male => body_fat_male(
            *measures.get("Waist").unwrap(),
            *measures.get("Neck").unwrap(),
            settings.height as f64,
        ),
        Sex::Female => body_fat_female(
            *measures.get("Waist").unwrap(),
            *measures.get("Neck").unwrap(),
            *measures.get("Hip").unwrap(),
            settings.height as f64,
        ),
    };

    let measure_id = get_measure_id(conn, "Body Fat")?;
    let id = get_measurement_id(conn, measure_id, date)?;

    // id option<i64>, id of existing body fat for the day
    // measure_id i64, body fat measure id
    match upsert_body_measurement_clean(conn, id, date, measure_id, bf) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn recompute_ffmi(
    conn: &rusqlite::Connection,
    settings: &Settings,
    date: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT m.name, b.value FROM body_measurements b
             JOIN body_metrics m ON b.measure_id = m.id
             WHERE b.date = ?1",
        )
        .map_err(|e| e.to_string())?;

    let measures: Vec<(String, f64)> = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut can_recompute_ffmi = true;
    can_recompute_ffmi &= measures.iter().any(|(m, _)| m == "Body Fat");
    can_recompute_ffmi &= measures.iter().any(|(m, _)| m == "Weight");

    let measures: HashMap<String, f64> = measures.into_iter().collect();
    if !can_recompute_ffmi {
        println!("Cannot recompute ffmi: missing required measurements");
        return Ok(());
    }
    // unwrap should be safe because we have checked that the fields exist, but feels a bit clunky
    let ffmi = calc_ffmi(
        *measures.get("Body Fat").unwrap(),
        *measures.get("Weight").unwrap(),
        settings.height as f64,
    );

    let measure_id = get_measure_id(conn, "FFMI")?;
    let id = get_measurement_id(conn, measure_id, date)?;

    match upsert_body_measurement_clean(conn, id, date, measure_id, ffmi) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn upsert_body_measurement_clean(
    conn: &rusqlite::Connection,
    id: Option<i64>,
    date: &str,
    measure_id: i64,
    value: f64,
) -> Result<i64, String> {
    let measurement_id: i64;
    if let Some(id) = id {
        measurement_id = id;
        conn.execute(
            "UPDATE body_measurements SET date = ?, measure_id = ?, value = ? WHERE id = ?",
            rusqlite::params![date, measure_id, value, measurement_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO body_measurements (date, measure_id, value) VALUES (?, ?, ?)",
            rusqlite::params![date, measure_id, value],
        )
        .map_err(|e| e.to_string())?;
        measurement_id = conn.last_insert_rowid();
    }
    Ok(measurement_id)
}

fn get_measure_id(conn: &rusqlite::Connection, measure: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT id FROM body_metrics WHERE name = ?1",
        rusqlite::params![measure],
        |row| Ok(row.get::<_, i64>(0)?),
    )
    .map_err(|e| e.to_string())
}

fn get_measurement_id(
    conn: &rusqlite::Connection,
    measure_id: i64,
    date: &str,
) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT id FROM body_measurements WHERE measure_id = ?1 AND date = ?2",
        rusqlite::params![measure_id, date],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn body_fat_female(waist_cm: f64, neck_cm: f64, hip_cm: f64, height_cm: f64) -> f64 {
    495.0
        / (1.29579 - 0.35004 * (waist_cm + hip_cm - neck_cm).log10() + 0.22100 * height_cm.log10())
        - 450.0
}

fn body_fat_male(waist_cm: f64, neck_cm: f64, height_cm: f64) -> f64 {
    495.0 / (1.0324 - 0.19077 * (waist_cm - neck_cm).log10() + 0.15456 * height_cm.log10()) - 450.0
}

fn calc_ffmi(body_fat_percentage: f64, weight_kg: f64, height_cm: f64) -> f64 {
    let fat_free_mass_kg = weight_kg * (1.0 - body_fat_percentage / 100.0);
    let height_m = height_cm / 100.0;

    fat_free_mass_kg / (height_m * height_m)
}
