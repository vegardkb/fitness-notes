use super::test_db;
use crate::commands::exercises::{
    get_rep_maxes_inner, list_exercise_categories_inner, list_exercises_in_category_inner,
};
use crate::commands::sets::upsert_set_inner;
use crate::commands::workouts::{add_exercise_to_workout_inner, get_active_dates_inner};
use crate::database::recompute_pr_flags;

#[test]
fn test_add_two_sets() {
    let conn = test_db();
    let categories = list_exercise_categories_inner(&conn).unwrap();
    let category = &categories[0];
    let exercises = list_exercises_in_category_inner(&conn, category.id).unwrap();
    let exercise = &exercises[0];
    let workout_exercise_id =
        add_exercise_to_workout_inner(&conn, "2024-01-01", exercise.id).unwrap();

    upsert_set_inner(&conn, None, workout_exercise_id, 100.0, 10, None).unwrap();
    upsert_set_inner(&conn, None, workout_exercise_id, 110.0, 5, None).unwrap();

    let rep_maxes = get_rep_maxes_inner(&conn, exercise.id).unwrap();

    assert_eq!(rep_maxes.len(), 2);
    assert_eq!(rep_maxes[0].reps, 5);
    assert_eq!(rep_maxes[0].weight_kg, 110.0);
    assert_eq!(rep_maxes[1].reps, 10);
    assert_eq!(rep_maxes[1].weight_kg, 100.0);

    let dates = get_active_dates_inner(&conn, 2024, 1).unwrap();
    assert_eq!(dates.len(), 1);
    assert_eq!(dates[0], "2024-01-01");

    let workout_exercise_id =
        add_exercise_to_workout_inner(&conn, "2024-01-11", exercise.id).unwrap();
    upsert_set_inner(&conn, None, workout_exercise_id, 105.0, 10, None).unwrap();

    let rep_maxes = get_rep_maxes_inner(&conn, exercise.id).unwrap();
    assert_eq!(rep_maxes.len(), 2);
    assert_eq!(rep_maxes[0].reps, 5);
    assert_eq!(rep_maxes[0].weight_kg, 110.0);
    assert_eq!(rep_maxes[1].reps, 10);
    assert_eq!(rep_maxes[1].weight_kg, 105.0);

    let dates = get_active_dates_inner(&conn, 2024, 1).unwrap();
    assert_eq!(dates.len(), 2);
    assert_eq!(dates[0], "2024-01-01");
    assert_eq!(dates[1], "2024-01-11");
}
