use crate::models::{
    DatedValue, DayMeasurement, DerivedMetricIds, Measurement, Metric, Settings, Sex,
};

use crate::commands::settings::get_settings_inner;

pub fn upsert_body_measurement_inner(
    conn: &rusqlite::Connection,
    id: Option<i64>,
    date: &str,
    measure_id: i64,
    value: f64,
) -> Result<i64, String> {
    let derived_ids = get_derived_metric_ids(conn)?;
    if derived_ids.bmi == measure_id
        || derived_ids.bf == measure_id
        || derived_ids.ffmi == measure_id
    {
        return Err("Can't insert or update derived metric".to_string());
    }

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

#[tauri::command]
pub fn upsert_body_measurement(
    id: Option<i64>,
    date: &str,
    measure_id: i64,
    value: f64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    upsert_body_measurement_inner(&conn, id, date, measure_id, value)
}

pub fn delete_body_measurement_inner(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM body_measurements WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_body_measurement(
    id: i64,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    delete_body_measurement_inner(&conn, id)
}

pub fn get_last_measurements_for_date_inner(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Vec<Measurement>, String> {
    let mut stmt = conn
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
        .map_err(|e| e.to_string())?;

    let rows = stmt
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
        .map_err(|e| e.to_string())?;

    let mut result: Vec<Measurement> = Vec::new();
    for row in rows {
        let (name, value, unit, metric_id, id, measure_date) = row.map_err(|e| e.to_string())?;

        result.push(Measurement {
            metric: Metric {
                name,
                unit,
                id: metric_id,
                is_derived: false,
            },
            value,
            date: Some(measure_date),
            id: Some(id),
        });
    }

    let settings = get_settings_inner(conn)?;
    let derived_metrics = get_derived_metric_ids(conn)?;
    let derived_result = calculate_derived_metrics(&settings, &result, &derived_metrics);
    for m in derived_result {
        result.push(m);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_last_measurements_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Measurement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_last_measurements_for_date_inner(&conn, date)
}

pub fn get_measurement_history_inner(
    conn: &rusqlite::Connection,
) -> Result<Vec<DayMeasurement>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bm.name, b.value, bm.unit, bm.id, b.id, b.date
             FROM body_measurements b
             JOIN body_metrics bm ON b.measure_id = bm.id
             ORDER BY b.date DESC, bm.name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
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
        .map_err(|e| e.to_string())?;

    let mut result: Vec<DayMeasurement> = Vec::new();
    for row in rows {
        let (metric_name, value, unit, metric_id, id, date) = row.map_err(|e| e.to_string())?;
        let metric = Metric {
            name: metric_name,
            unit,
            id: metric_id,
            is_derived: false,
        };
        let measurement = Measurement {
            metric,
            value,
            date: Some(date.clone()),
            id: Some(id),
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

    let derived_metrics = get_derived_metric_ids(conn)?;
    let settings = get_settings_inner(conn)?;
    for day in &mut result {
        let derived_result =
            calculate_derived_metrics(&settings, &day.measurements, &derived_metrics);
        for m in derived_result {
            day.measurements.push(m);
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_measurement_history(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DayMeasurement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_measurement_history_inner(&conn)
}

pub fn get_measurements_for_date_inner(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Vec<Measurement>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bm.name, b.value, bm.unit, bm.id, b.id
             FROM body_measurements b
             JOIN body_metrics bm ON b.measure_id = bm.id
             WHERE b.date = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, String>(0)?, // metric name
                row.get::<_, f64>(1)?,    // value
                row.get::<_, String>(2)?, // unit of measurement
                row.get::<_, i64>(3)?,    // metric id
                row.get::<_, i64>(4)?,    // measurement id
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<Measurement> = Vec::new();
    for row in rows {
        let (name, value, unit, metric_id, id) = row.map_err(|e| e.to_string())?;

        result.push(Measurement {
            metric: Metric {
                name,
                unit,
                id: metric_id,
                is_derived: false,
            },
            value,
            date: Some(date.to_string()),
            id: Some(id),
        });
    }
    let settings = get_settings_inner(conn)?;
    let derived_metrics = get_derived_metric_ids(conn)?;
    let derived_result = calculate_derived_metrics(&settings, &result, &derived_metrics);
    for m in derived_result {
        result.push(m);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_measurements_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Measurement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_measurements_for_date_inner(&conn, date)
}

pub fn get_measurements_graph_data_inner(
    conn: &rusqlite::Connection,
    metric_id: i64,
    from_date: &str,
    to_date: &str,
) -> Result<Vec<DatedValue>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bm.name, b.value, bm.unit, bm.id, b.id, b.date
                FROM body_measurements b
                JOIN body_metrics bm ON b.measure_id = bm.id
                WHERE b.date BETWEEN ?1 AND ?2
                ORDER BY b.date DESC, bm.name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![from_date, to_date], |row| {
            Ok((
                row.get::<_, String>(0)?, // metric name
                row.get::<_, f64>(1)?,    // value
                row.get::<_, String>(2)?, // unit of measurement
                row.get::<_, i64>(3)?,    // metric id
                row.get::<_, i64>(4)?,    // body measurement id
                row.get::<_, String>(5)?, // date
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<DayMeasurement> = Vec::new();
    for row in rows {
        let (metric_name, value, unit, metric_id, id, date) = row.map_err(|e| e.to_string())?;
        let metric = Metric {
            name: metric_name,
            unit,
            id: metric_id,
            is_derived: false,
        };
        let measurement = Measurement {
            metric,
            value,
            date: Some(date.clone()),
            id: Some(id),
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

    let derived_metrics = get_derived_metric_ids(conn)?;
    let settings = get_settings_inner(conn)?;
    for day in &mut result {
        let derived_result =
            calculate_derived_metrics(&settings, &day.measurements, &derived_metrics);
        for m in derived_result {
            day.measurements.push(m);
        }
    }
    let result = result
        .iter()
        .filter(|d| {
            d.date >= from_date.to_string()
                && d.date <= to_date.to_string()
                && d.measurements.iter().any(|m| m.metric.id == metric_id)
        })
        .map(|d| DatedValue {
            date: d.date.clone(),
            value: d
                .measurements
                .iter()
                .find(|m| m.metric.id == metric_id)
                .map(|m| m.value)
                .unwrap_or_default(),
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub fn get_measurements_graph_data(
    metric_id: i64,
    from_date: &str,
    to_date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DatedValue>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_measurements_graph_data_inner(&conn, metric_id, from_date, to_date)
}

pub fn list_metrics_inner(conn: &rusqlite::Connection) -> Result<Vec<Metric>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bm.name, bm.unit, bm.id, bm.is_derived FROM body_metrics bm
             ORDER BY bm.name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(Metric {
                name: row.get::<_, String>(0)?,
                unit: row.get::<_, String>(1)?,
                id: row.get::<_, i64>(2)?,
                is_derived: row.get::<_, bool>(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn list_metrics(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<Metric>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    list_metrics_inner(&conn)
}

fn calculate_derived_metrics(
    settings: &Settings,
    result: &[Measurement],
    derived_ids: &DerivedMetricIds,
) -> Vec<Measurement> {
    let weight = result
        .iter()
        .find(|m| m.metric.name == "Weight")
        .map(|m| m.value);
    let height = settings.height as f64;

    let mut derived_result = Vec::new();

    if let Some(weight) = weight {
        let bmi = calc_bmi(weight, height);
        derived_result.push(Measurement {
            metric: Metric {
                name: "BMI".to_string(),
                unit: "kg/m²".to_string(),
                id: derived_ids.bmi,
                is_derived: true,
            },
            value: bmi,
            date: get_derived_metric_date(result, "BMI", &settings.sex),
            id: None,
        });
    }

    let waist = result
        .iter()
        .find(|m| m.metric.name == "Waist")
        .map(|m| m.value);
    let neck = result
        .iter()
        .find(|m| m.metric.name == "Neck")
        .map(|m| m.value);
    let hip = result
        .iter()
        .find(|m| m.metric.name == "Hip")
        .map(|m| m.value);
    let bf: Option<f64> = match settings.sex {
        Sex::Female => match (waist, neck, hip) {
            (Some(waist), Some(neck), Some(hip)) => Some(body_fat_female(waist, neck, hip, height)),
            _ => None,
        },
        Sex::Male => match (waist, neck) {
            (Some(waist), Some(neck)) => Some(body_fat_male(waist, neck, height)),
            _ => None,
        },
    };
    if let Some(bf) = bf {
        derived_result.push(Measurement {
            metric: Metric {
                name: "Body Fat (Navy)".to_string(),
                unit: "%".to_string(),
                id: derived_ids.bf,
                is_derived: true,
            },
            value: bf,
            date: get_derived_metric_date(result, "Body Fat (Navy)", &settings.sex),
            id: None,
        });
        if let Some(weight) = weight {
            let ffmi = calc_ffmi(bf, weight, height);
            derived_result.push(Measurement {
                metric: Metric {
                    name: "FFMI (Navy)".to_string(),
                    unit: "kg/m²".to_string(),
                    id: derived_ids.ffmi,
                    is_derived: true,
                },
                value: ffmi,
                date: get_derived_metric_date(result, "FFMI (Navy)", &settings.sex),
                id: None,
            });
        }
    }
    derived_result
}

fn get_derived_metric_date(result: &[Measurement], metric_name: &str, sex: &Sex) -> Option<String> {
    let measurement = match metric_name {
        "BMI" => result
            .iter()
            .filter(|m| m.metric.name == "Weight")
            .max_by(|x, y| x.date.cmp(&y.date)),
        _ => match sex {
            Sex::Male => match metric_name {
                "Body Fat (Navy)" => result
                    .iter()
                    .filter(|m| m.metric.name == "Neck" || m.metric.name == "Waist")
                    .max_by(|x, y| x.date.cmp(&y.date)),
                "FFMI (Navy)" => result
                    .iter()
                    .filter(|m| {
                        m.metric.name == "Weight"
                            || m.metric.name == "Neck"
                            || m.metric.name == "Waist"
                    })
                    .max_by(|x, y| x.date.cmp(&y.date)),
                _ => None,
            },
            Sex::Female => match metric_name {
                "Body Fat (Navy)" => result
                    .iter()
                    .filter(|m| {
                        m.metric.name == "Neck"
                            || m.metric.name == "Waist"
                            || m.metric.name == "Hip"
                    })
                    .max_by(|x, y| x.date.cmp(&y.date)),
                "FFMI (Navy)" => result
                    .iter()
                    .filter(|m| {
                        m.metric.name == "Weight"
                            || m.metric.name == "Neck"
                            || m.metric.name == "Waist"
                            || m.metric.name == "Hip"
                    })
                    .max_by(|x, y| x.date.cmp(&y.date)),
                _ => None,
            },
        },
    };
    match measurement {
        Some(measurement) => measurement.clone().date,
        None => None,
    }
}

fn get_derived_metric_ids(conn: &rusqlite::Connection) -> Result<DerivedMetricIds, String> {
    let mut stmt = conn
        .prepare("SELECT name, id FROM body_metrics WHERE is_derived = true")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?, // metric name
                row.get::<_, i64>(1)?,    // metric id
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut ids = DerivedMetricIds {
        bmi: 0,
        bf: 0,
        ffmi: 0,
    };
    for row in rows {
        let (name, id) = match row {
            Ok(row) => (row.0, row.1),
            Err(e) => {
                return Err(e.to_string());
            }
        };
        match name.as_str() {
            "BMI" => ids.bmi = id,
            "Body Fat (Navy)" => ids.bf = id,
            "FFMI (Navy)" => ids.ffmi = id,
            _ => (),
        }
    }
    if ids.bmi == 0 || ids.bf == 0 || ids.ffmi == 0 {
        return Err("Missing derived metric rows in body_metrics".into());
    }
    Ok(ids)
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

fn calc_bmi(weight_kg: f64, height_cm: f64) -> f64 {
    let height_m = height_cm / 100.0;

    weight_kg / (height_m * height_m)
}
