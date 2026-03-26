export type Set = {
  id: number;
  set_order: number;
  weight_kg: number;
  reps: number;
  notes: string | null;
  was_pr_at_time: boolean;
  is_current_pr: boolean;
};

export type ExerciseWithSets = {
  id: number; // alias of exercise_id, required by dndzone
  exercise_id: number;
  exercise_name: string;
  sets: Set[];
};

export type Exercise = {
  id: number;
  name: string;
};

export type DayWorkout = {
  date: string;
  exercises: ExerciseWithSets[];
};

export function formatWeight(kg: number): string {
  const f2 = kg.toFixed(2);
  return f2.endsWith("0") ? kg.toFixed(1) : f2;
}
