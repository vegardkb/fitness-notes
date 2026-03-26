use serde::Serialize;

#[derive(Serialize)]
pub struct Exercise {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct ExerciseWithSets {
    pub exercise_id: i64,
    pub exercise_name: String,
    pub category: String,
    pub exercise_order: i64,
    pub sets: Vec<Set>,
}

#[derive(Serialize)]
pub struct DayWorkout {
    pub date: String,
    pub exercises: Vec<ExerciseWithSets>,
}

#[derive(Serialize)]
pub struct Set {
    pub id: i64,
    pub set_order: i64,
    pub reps: i64,
    pub weight_kg: f64,
    pub notes: Option<String>,
    pub was_pr_at_time: bool,
    pub is_current_pr: bool,
}

#[derive(Serialize)]
pub struct DatedValue {
    pub date: String,
    pub metric: f64,
}

#[derive(Serialize)]
pub struct RepMax {
    pub date: String,
    pub reps: i64,
    pub weight_kg: f64,
}
