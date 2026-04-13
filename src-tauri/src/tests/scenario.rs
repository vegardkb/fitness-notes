use super::test_db;
use crate::commands::exercises::{
    get_exercise_history_inner, get_rep_maxes_inner, list_exercise_categories_inner,
    list_exercises_in_category_inner,
};
use crate::commands::sets::upsert_set_inner;
use crate::commands::workouts::{
    add_exercise_to_workout_inner, get_active_dates_inner, get_sets_for_workout_exercise_inner,
};

#[test]
fn test_add_three_sets_on_two_days() {
    let conn = test_db();
    let categories = list_exercise_categories_inner(&conn).unwrap();
    let category = &categories[0];
    let exercises = list_exercises_in_category_inner(&conn, category.id).unwrap();
    let exercise = &exercises[0];
    let workout_exercise_id =
        add_exercise_to_workout_inner(&conn, "2024-01-01", exercise.id).unwrap();

    let sets = get_sets_for_workout_exercise_inner(&conn, workout_exercise_id).unwrap();
    assert_eq!(sets.len(), 0);

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

    let exercise_history = get_exercise_history_inner(&conn, exercise.id).unwrap();
    assert_eq!(exercise_history.len(), 2);
    assert_eq!(exercise_history[1].date, "2024-01-01");
    assert_eq!(exercise_history[0].date, "2024-01-11");

    assert_eq!(
        exercise_history[1].exercises[0].sets[0].is_current_pr,
        false
    );
    assert_eq!(
        exercise_history[1].exercises[0].sets[0].was_pr_at_time,
        true
    );
    assert_eq!(exercise_history[1].exercises[0].sets[1].is_current_pr, true);
    assert_eq!(
        exercise_history[1].exercises[0].sets[1].was_pr_at_time,
        true
    );
    assert_eq!(exercise_history[0].exercises[0].sets[0].is_current_pr, true);
    assert_eq!(
        exercise_history[0].exercises[0].sets[0].was_pr_at_time,
        true
    );
}

#[test]
fn migrations_are_idempotent() {
    let conn = test_db();
    crate::database::run_migrations(&conn).unwrap();
}

#[test]
fn schema_version_is_current() {
    let conn = test_db();
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 4);
}
