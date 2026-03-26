use crate::models::{DayWorkout, ExerciseWithSets, Set};

#[tauri::command]
pub fn get_workout_for_date(
    date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<ExerciseWithSets>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name, c.name, we.exercise_order,
                    s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN categories c ON e.category_id = c.id
             JOIN sets s ON s.workout_id = w.id AND s.exercise_id = e.id
             WHERE w.date = ?1
             ORDER BY we.exercise_order, s.set_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, i64>(0)?,            // exercise_id
                row.get::<_, String>(1)?,         // exercise_name
                row.get::<_, String>(2)?,         // category
                row.get::<_, i64>(3)?,            // exercise_order
                row.get::<_, i64>(4)?,            // set id
                row.get::<_, i64>(5)?,            // set_order
                row.get::<_, f64>(6)?,            // weight_kg
                row.get::<_, i64>(7)?,            // reps
                row.get::<_, Option<String>>(8)?, // notes
                row.get::<_, bool>(9)?,           // was_pr_at_time
                row.get::<_, bool>(10)?,          // is_current_pr
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<ExerciseWithSets> = Vec::new();
    for row in rows {
        let (
            exercise_id,
            exercise_name,
            category,
            exercise_order,
            set_id,
            set_order,
            weight_kg,
            reps,
            notes,
            was_pr_at_time,
            is_current_pr,
        ) = row.map_err(|e| e.to_string())?;

        let set = Set {
            id: set_id,
            set_order,
            weight_kg,
            reps,
            notes,
            was_pr_at_time,
            is_current_pr,
        };

        match result.last_mut() {
            Some(last) if last.exercise_id == exercise_id => last.sets.push(set),
            _ => result.push(ExerciseWithSets {
                exercise_id,
                exercise_name,
                category,
                exercise_order,
                sets: vec![set],
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_workouts_for_range(
    from_date: &str,
    to_date: &str,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<DayWorkout>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT w.date, e.id, e.name, c.name, we.exercise_order,
                    s.id, s.set_order, s.weight_kg, s.reps, s.notes, s.was_pr_at_time, s.is_current_pr
             FROM workouts w
             JOIN workout_exercises we ON we.workout_id = w.id
             JOIN exercises e ON we.exercise_id = e.id
             JOIN categories c ON e.category_id = c.id
             JOIN sets s ON s.workout_id = w.id AND s.exercise_id = e.id
             WHERE w.date BETWEEN ?1 AND ?2
             ORDER BY w.date DESC, we.exercise_order, s.set_order",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![from_date, to_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, f64>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, bool>(10)?,
                row.get::<_, bool>(11)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result: Vec<DayWorkout> = Vec::new();
    for row in rows {
        let (
            date,
            exercise_id,
            exercise_name,
            category,
            exercise_order,
            set_id,
            set_order,
            weight_kg,
            reps,
            notes,
            was_pr_at_time,
            is_current_pr,
        ) = row.map_err(|e| e.to_string())?;

        let set = Set {
            id: set_id,
            set_order,
            weight_kg,
            reps,
            notes,
            was_pr_at_time,
            is_current_pr,
        };

        let day = match result.last_mut() {
            Some(d) if d.date == date => d,
            _ => {
                result.push(DayWorkout {
                    date: date.clone(),
                    exercises: Vec::new(),
                });
                result.last_mut().unwrap()
            }
        };

        match day.exercises.last_mut() {
            Some(ex) if ex.exercise_id == exercise_id => ex.sets.push(set),
            _ => day.exercises.push(ExerciseWithSets {
                exercise_id,
                exercise_name,
                category,
                exercise_order,
                sets: vec![set],
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_active_dates(
    year: i32,
    month: u32,
    db: tauri::State<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<Vec<String>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let prefix = format!("{}-{:02}", year, month);
    let mut stmt = conn
        .prepare("SELECT date FROM workouts WHERE date LIKE ?1 ORDER BY date")
        .map_err(|e| e.to_string())?;
    let dates = stmt
        .query_map(rusqlite::params![format!("{}%", prefix)], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(dates)
}
