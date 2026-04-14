use crate::models::{Settings, Sex, WeightUnit};

pub fn delete_all_data_inner(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "DELETE FROM sets;
         DELETE FROM workout_exercises;
         DELETE FROM workouts;
         DELETE FROM body_measurements;",
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_all_data(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    delete_all_data_inner(&conn)
}

pub fn get_settings_inner(conn: &rusqlite::Connection) -> Result<Settings, String> {
    let mut stmt = conn
        .prepare("SELECT us.height_cm, us.sex, us.dark_mode, us.unit FROM user_settings us")
        .map_err(|e| e.to_string())?;

    let mut row = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let r = row.next();
    match r {
        Some(Ok((height, sex, dark_mode, unit))) => Ok(Settings {
            height,
            sex: Sex::from(sex),
            dark_mode,
            unit: WeightUnit::from(unit),
        }),
        Some(Err(e)) => Err(e.to_string()),
        None => Err("No settings found".to_string()),
    }
}

#[tauri::command]
pub fn get_settings(
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Settings, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    get_settings_inner(&conn)
}
