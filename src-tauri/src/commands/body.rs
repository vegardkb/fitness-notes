use crate::models::Sex;
use rusqlite::OptionalExtension;
use std::collections::HashMap;

use crate::{database::get_settings, models::Settings};

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

    fat_free_mass_kg / (height_cm * height_cm)
}

#[tauri::command]
pub fn upsert_body_measurement(
    id: Option<i64>,
    date: &str,
    measure_id: i64,
    value: f64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    // This block will be repeated 3 times, may turn into a function
    upsert_body_measurement_clean(&conn, id, date, measure_id, value)?;

    // Recompute derived metrics after upsert
    // Recompute body fat percentage if measure_id corresponds to (waist_cm, neck_cm) if male or (waist_cm, hip_cm, neck_cm) if female
    // Recompute ffmi if weight or body fat percentage is upserted
    let measure = conn
        .query_row(
            "SELECT name FROM body_metrics WHERE id = ?1",
            rusqlite::params![measure_id],
            |row| Ok(row.get::<_, String>(0)?),
        )
        .map_err(|e| e.to_string())?;

    // query settings to get height, sex and body fat percentage preference
    let settings = get_settings(&conn)?;

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

    Ok(())
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
        return Err(
            "Cannot recompute body fat: missing required measurements or permissions".to_string(),
        );
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
    upsert_body_measurement_clean(conn, id, date, measure_id, bf)
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
        return Err("Cannot recompute ffmi: missing required measurements".to_string());
    }
    // unwrap should be safe because we have checked that the fields exist, but feels a bit clunky
    let ffmi = calc_ffmi(
        *measures.get("Body Fat").unwrap(),
        *measures.get("Weight").unwrap(),
        settings.height as f64,
    );

    let measure_id = get_measure_id(conn, "FFMI")?;
    let id = get_measurement_id(conn, measure_id, date)?;

    upsert_body_measurement_clean(conn, id, date, measure_id, ffmi)
}

fn upsert_body_measurement_clean(
    conn: &rusqlite::Connection,
    id: Option<i64>,
    date: &str,
    measure_id: i64,
    value: f64,
) -> Result<(), String> {
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
    };
    Ok(())
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
