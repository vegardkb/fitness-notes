import { todayStr } from "$lib/date";

export type Set = {
  id: number;
  set_order: number;
  weight_kg: number;
  reps: number;
  notes: string | null;
  was_pr_at_time: boolean;
  is_current_pr: boolean;
};

export type SetMinimal = {
  weight: number;
  reps: number;
};

export type RepMax = {
  date: string;
  weight_kg: number;
  reps: number;
};

export type ExerciseWithSets = {
  id: number; // alias of exercise_id, required by dndzone
  workout_exercise_id: number;
  exercise_id: number;
  exercise_name: string;
  sets: Set[];
};

export type Category = {
  id: number;
  name: string;
};

export type Exercise = {
  id: number;
  name: string;
};

export type DayWorkout = {
  date: string;
  exercises: ExerciseWithSets[];
};

export type WorkoutExerciseContext = {
  exercise_name: string;
  date: string;
};

export function formatWeight(kg: number): string {
  const f2 = kg.toFixed(2);
  return f2.endsWith("0") ? kg.toFixed(1) : f2;
}

export function exerciseHrefs(
  exerciseId: number,
  workoutExerciseId: number,
  fromDate: string,
) {
  const from = workoutExerciseId ? `?from=${workoutExerciseId}` : "";
  return {
    feedHref: fromDate ? `/?date=${fromDate}` : "/",
    setsHref: `/exercise/${exerciseId}/${workoutExerciseId}`,
    historyHref: `/exercise/${exerciseId}/history${from}`,
    graphHref: `/exercise/${exerciseId}/graph${from}`,
    prsHref: `/exercise/${exerciseId}/prs${from}`,
  };
}
