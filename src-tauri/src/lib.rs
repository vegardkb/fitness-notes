use crate::commands::exercises::{
    get_exercise, get_exercise_graph_data, get_exercise_history, get_rep_maxes,
    list_exercise_categories, list_exercises_in_category,
};
use crate::commands::import::{import_fitnotes_rows, parse_fitnotes_csv};
use crate::commands::sets::{delete_set, reorder_exercises, reorder_sets, upsert_set};
use crate::commands::workouts::{get_active_dates, get_workout_for_date, get_workouts_for_range};

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
            get_active_dates,
            get_workout_for_date,
            upsert_set,
            delete_set,
            get_exercise_history,
            get_exercise_graph_data,
            list_exercise_categories,
            list_exercises_in_category,
            get_exercise,
            get_rep_maxes,
            reorder_exercises,
            reorder_sets,
            get_workouts_for_range,
            parse_fitnotes_csv,
            import_fitnotes_rows,
            delete_all_data,
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
         DELETE FROM workouts;",
    )
    .map_err(|e| e.to_string())
}

fn initialize_db(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    let conn = rusqlite::Connection::open(app_data_dir.join("fitness_notes.db"))?;

    database::create_tables(&conn)?;
    app.manage(std::sync::Mutex::new(conn));
    Ok(())
}
