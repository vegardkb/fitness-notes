use serde::Serialize;

#[derive(Serialize)]
pub struct Exercise {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct Category {
    pub name: String,
    pub id: i64,
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
pub struct SetMinimal {
    pub weight: f64,
    pub reps: i64,
}

#[derive(Serialize)]
pub struct DatedValue {
    pub date: String,
    pub value: f64,
}

#[derive(Serialize)]
pub struct RepMax {
    pub date: String,
    pub reps: i64,
    pub weight_kg: f64,
}

pub struct Settings {
    pub height: i64,
    pub unit: WeightUnit,
    pub dark_mode: bool,
    pub sex: Sex,
}

#[derive(Serialize)]
pub struct DayMeasurement {
    pub date: String,
    pub measurements: Vec<Measurement>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Measurement {
    pub metric: Metric,
    pub value: f64,
    pub date: Option<String>,
    pub id: Option<i64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub unit: String,
    pub id: i64,
    pub is_derived: bool,
}

pub struct DerivedMetricIds {
    pub bmi: i64,
    pub bf: i64,
    pub ffmi: i64,
}

pub enum Sex {
    Male,
    Female,
}

impl From<String> for Sex {
    fn from(s: String) -> Self {
        match s.as_str() {
            "male" => Self::Male,
            "female" => Self::Female,
            _ => Self::Male,
        }
    }
}

pub enum WeightUnit {
    Kg,
    Lbs,
}

impl From<String> for WeightUnit {
    fn from(s: String) -> Self {
        match s.as_str() {
            "kg" => Self::Kg,
            "lbs" => Self::Lbs,
            _ => Self::Kg,
        }
    }
}
