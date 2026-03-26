use crate::models::Sex;
use std::{collections::HashMap, sync::Mutex};

use crate::{database::get_settings, models::Settings};

fn body_fat_female(waist_cm: f64, neck_cm: f64, hip_cm: f64, height_cm: f64) -> f64 {
    495.0
        / (1.29579 - 0.35004 * (waist_cm + hip_cm - neck_cm).log10() + 0.22100 * height_cm.log10())
        - 450.0
}

fn body_fat_male(waist_cm: f64, neck_cm: f64, height_cm: f64) -> f64 {
    495.0 / (1.0324 - 0.19077 * (waist_cm - neck_cm).log10() + 0.15456 * height_cm.log10()) - 450.0
}

fn ffmi(body_fat_percentage: f64, weight_kg: f64, height_cm: f64) -> f64 {
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
    }

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
    let settings = get_settings(conn)?;

    let mut should_recompute_bf = false;
    if measure == "Waist" || measure == "Neck" {
        should_recompute_bf = true;
    } else if measure == "Hip" && settings.sex == Sex::Female {
        should_recompute_bf = true;
    }
    should_recompute_bf = should_recompute_bf && settings.2;

    if should_recompute_bf {
        recompute_body_fat(conn, measure.as_str(), &settings, date);
    }

    // Fetch necessary measures for body fat calc and ffmi calc
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
    if settings.1 == "Female" {
        can_recompute_body_fat &= measures.iter().any(|(m, _)| m == "Hip");
    }

    let measures: HashMap<String, f64> = measures.into_iter().collect();
    let bf: f64;
    if recompute_body_fat && can_recompute_body_fat {
        let bf = match sex {
            Sex::Male => body_fat_male(
                *measures.get("Waist").unwrap(),
                *measures.get("Neck").unwrap(),
                settings.0 as f64,
            ),
            Sex::Female => body_fat_female(
                *measures.get("Waist").unwrap(),
                *measures.get("Neck").unwrap(),
                *measures.get("Hip").unwrap(),
                settings.0 as f64,
            ),
        };
    }

    let recompute_ffmi = recompute_body_fat || measure == "Weight" || measure == "Body Fat";

    Ok(())
}

fn recompute_body_fat(
    conn: &rusqlite::Connection,
    measure: &str,
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
    let bf: f64;
    if !can_recompute_body_fat {
        return Err(
            "Cannot recompute body fat: missing required measurements or permissions".to_string(),
        );
    }
    let bf = match settings.sex {
        Sex::Male => body_fat_male(
            *measures.get("Waist").unwrap(),
            *measures.get("Neck").unwrap(),
            settings.0 as f64,
        ),
        Sex::Female => body_fat_female(
            *measures.get("Waist").unwrap(),
            *measures.get("Neck").unwrap(),
            *measures.get("Hip").unwrap(),
            settings.0 as f64,
        ),
    };

    Ok(())
}
