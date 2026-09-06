// Deterministic fixtures for WebdriverIO desktop tests. Only compiled with
// the `wdio` cargo feature: the test binary wipes all user data and inserts
// a fixed workout on every boot, so runs are reproducible on any machine and
// never touch the real database (the test build also runs under a separate
// identifier via tauri.wdio.conf.json).
use rusqlite::Connection;

pub fn reset_and_seed(conn: &Connection) -> Result<(), String> {
    // Wipe everything except user_settings and body_metrics (reference data
    // the app expects to exist).
    conn.execute_batch("PRAGMA foreign_keys = OFF;")
        .map_err(|e| e.to_string())?;
    conn.execute_batch(
        "DELETE FROM sets;
         DELETE FROM workout_exercises;
         DELETE FROM workouts;
         DELETE FROM exercises;
         DELETE FROM categories;
         DELETE FROM template_sets;
         DELETE FROM template_exercises;
         DELETE FROM templates;
         DELETE FROM body_measurements;",
    )
    .map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;

    // One workout two days ago: Bench Press 80x5, 90x5, 100x5.
    // ids are deterministic (1, 1, 1-3) because the tables were emptied.
    let two_days_ago = days_ago(2);
    conn.execute_batch(&format!(
        "INSERT INTO categories (id, name) VALUES (1, 'Chest');
        INSERT INTO exercises (id, name, category_id) VALUES (1, 'Bench Press', 1);
        INSERT INTO exercises (id, name, category_id) VALUES (2, 'DB Bench Press', 1);
        INSERT INTO workouts (id, date, workout_order, name)
            VALUES (1, '{two_days_ago}', 1, 'Workout');
        INSERT INTO workout_exercises (id, workout_id, exercise_id, exercise_order)
            VALUES (1, 1, 1, 1);
        INSERT INTO sets (id, workout_exercise_id, exercise_id, set_order,
                        weight_kg, reps, notes, was_pr_at_time, is_current_pr)
        VALUES (1, 1, 1, 1, 80.0, 5, NULL, false, false),
            (2, 1, 1, 2, 90.0, 5, NULL, false, false),
            (3, 1, 1, 3, 100.0, 5, NULL, false, false);
        INSERT INTO workout_exercises (id, workout_id, exercise_id, exercise_order)
            VALUES (2, 1, 2, 2);
        INSERT INTO sets (id, workout_exercise_id, exercise_id, set_order,
                            weight_kg, reps, notes, was_pr_at_time, is_current_pr)
        VALUES (4, 2, 2, 1, 40.0, 8, NULL, false, false),
            (5, 2, 2, 2, 50.0, 8, NULL, false, false);"
    ))
    .map_err(|e| e.to_string())?;

    crate::database::recompute_pr_flags(conn, 1)?;
    Ok(())
}

fn days_ago(n: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86400
        - n;
    crate::gregorian_from_days(days)
}
