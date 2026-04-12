use crate::commands::body::{
    delete_body_measurement, get_last_measurements_for_date, get_measurement_history,
    get_measurements_for_date, list_metrics, upsert_body_measurement,
};
use crate::commands::exercises::{
    create_category, create_exercise, delete_category, delete_exercise, get_exercise,
    get_exercise_graph_data, get_exercise_history, get_last_set, get_rep_maxes,
    list_exercise_categories, list_exercises_in_category, merge_category_into_existing,
    merge_exercise_into_existing, rename_category, rename_exercise,
};
use crate::commands::import::{
    import_body_measurement_rows, import_fitnotes_rows, parse_body_measurements_csv,
    parse_fitnotes_csv,
};
use crate::commands::sets::{delete_set, reorder_exercises, reorder_sets, upsert_set};
use crate::commands::workouts::{
    add_exercise_to_workout, get_active_dates, get_workout_for_date, get_workouts_for_range,
    remove_exercise_from_workout,
};

use tauri::Manager;

mod commands;
mod database;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            initialize_db(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_exercise_to_workout,
            remove_exercise_from_workout,
            get_active_dates,
            get_workout_for_date,
            upsert_set,
            delete_set,
            get_exercise_history,
            get_exercise_graph_data,
            list_exercise_categories,
            list_exercises_in_category,
            create_exercise,
            delete_exercise,
            rename_exercise,
            merge_exercise_into_existing,
            merge_category_into_existing,
            create_category,
            delete_category,
            rename_category,
            get_exercise,
            get_rep_maxes,
            get_last_set,
            reorder_exercises,
            reorder_sets,
            get_workouts_for_range,
            parse_fitnotes_csv,
            import_fitnotes_rows,
            parse_body_measurements_csv,
            import_body_measurement_rows,
            delete_all_data,
            upsert_body_measurement,
            delete_body_measurement,
            get_measurements_for_date,
            get_last_measurements_for_date,
            get_measurement_history,
            list_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn delete_all_data(db: tauri::State<std::sync::Mutex<rusqlite::Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "DELETE FROM sets;
         DELETE FROM workout_exercises;
         DELETE FROM workouts;
         DELETE FROM body_measurements;",
    )
    .map_err(|e| e.to_string())
}

fn initialize_db(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("fitness_notes.db");

    if db_path.exists() {
        backup_db(&app_data_dir, &db_path)?;
    }

    let conn = rusqlite::Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    database::run_migrations(&conn)?;
    app.manage(std::sync::Mutex::new(conn));
    Ok(())
}

fn backup_db(
    app_data_dir: &std::path::Path,
    db_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let backups_dir = app_data_dir.join("backups");
    std::fs::create_dir_all(&backups_dir)?;

    let backup_path = backups_dir.join(format!("{}.db", today_str()));
    if !backup_path.exists() {
        // Checkpoint WAL into the main file before copying so the backup is self-contained
        let tmp = rusqlite::Connection::open(db_path)?;
        tmp.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        drop(tmp);
        std::fs::copy(db_path, &backup_path)?;
    }

    // Keep the 14 most recent backups, delete the rest
    let mut entries: Vec<_> = std::fs::read_dir(&backups_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "db"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries.iter().take(entries.len().saturating_sub(14)) {
        std::fs::remove_file(entry.path())?;
    }

    Ok(())
}

fn today_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86400;
    // Gregorian calendar from days since Unix epoch (Howard Hinnant's algorithm)
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}
