import type { InvokeArgs } from "@tauri-apps/api/core";

// Deterministic mock backend used when the frontend is served outside the
// Tauri webview (e.g. `pnpm dev` opened in a regular browser for screenshot
// tooling). Only `get_*` and `list_*` are implemented — mutations no-op.

const TODAY = "2026-04-19";

function daysAgo(n: number): string {
  const d = new Date(TODAY + "T00:00:00Z");
  d.setUTCDate(d.getUTCDate() - n);
  return d.toISOString().slice(0, 10);
}

// --- Static reference data ---

const CATEGORIES = [
  { id: 1, name: "Chest" },
  { id: 2, name: "Back" },
  { id: 3, name: "Legs" },
  { id: 4, name: "Shoulders" },
  { id: 5, name: "Arms" },
];

const EXERCISES = [
  { id: 1, name: "Bench Press", category_id: 1 },
  { id: 2, name: "Incline Dumbbell Press", category_id: 1 },
  { id: 3, name: "Barbell Row", category_id: 2 },
  { id: 4, name: "Pull-Up", category_id: 2 },
  { id: 5, name: "Back Squat", category_id: 3 },
  { id: 6, name: "Romanian Deadlift", category_id: 3 },
  { id: 7, name: "Overhead Press", category_id: 4 },
  { id: 8, name: "Barbell Curl", category_id: 5 },
];

const categoryName = (id: number) =>
  CATEGORIES.find((c) => c.id === id)?.name ?? "Uncategorized";

// --- Workout fixture builder ---

type MockSet = {
  id: number;
  set_order: number;
  reps: number;
  weight_kg: number;
  notes: string | null;
  was_pr_at_time: boolean;
  is_current_pr: boolean;
};

type MockWorkoutExercise = {
  workout_exercise_id: number;
  exercise_id: number;
  exercise_order: number;
  sets: MockSet[];
};

type MockWorkout = {
  id: number;
  date: string;
  name: string | null;
  exercises: MockWorkoutExercise[];
};

type SetSpec = { w: number; r: number; pr?: boolean; notes?: string };
type ExerciseSpec = { exerciseId: number; sets: SetSpec[] };

const WORKOUTS: MockWorkout[] = (() => {
  let weCounter = 1;
  let setCounter = 1;
  const out: MockWorkout[] = [];

  const add = (date: string, name: string | null, entries: ExerciseSpec[]) => {
    out.push({
      id: out.length + 1,
      date,
      name,
      exercises: entries.map((e, i) => ({
        workout_exercise_id: weCounter++,
        exercise_id: e.exerciseId,
        exercise_order: i + 1,
        sets: e.sets.map((s, j) => ({
          id: setCounter++,
          set_order: j + 1,
          reps: s.r,
          weight_kg: s.w,
          notes: s.notes ?? null,
          was_pr_at_time: false,
          is_current_pr: !!s.pr,
        })),
      })),
    });
  };

  add(daysAgo(2), "Push day", [
    {
      exerciseId: 1,
      sets: [
        { w: 80, r: 5 },
        { w: 90, r: 5 },
        { w: 102.5, r: 5, pr: true, notes: "Felt strong" },
      ],
    },
    {
      exerciseId: 7,
      sets: [
        { w: 50, r: 6 },
        { w: 55, r: 5 },
        { w: 60, r: 3, pr: true },
      ],
    },
    {
      exerciseId: 8,
      sets: [
        { w: 25, r: 10 },
        { w: 27.5, r: 8 },
        { w: 27.5, r: 8 },
      ],
    },
  ]);

  add(daysAgo(4), "Pull day", [
    {
      exerciseId: 3,
      sets: [
        { w: 70, r: 8 },
        { w: 80, r: 8 },
        { w: 85, r: 6, pr: true },
      ],
    },
    {
      exerciseId: 4,
      sets: [
        { w: 0, r: 10 },
        { w: 0, r: 8 },
        { w: 10, r: 6, pr: true, notes: "Added dip belt" },
      ],
    },
  ]);

  add(daysAgo(6), "Leg day", [
    {
      exerciseId: 5,
      sets: [
        { w: 100, r: 5 },
        { w: 120, r: 5 },
        { w: 135, r: 3, pr: true },
      ],
    },
    {
      exerciseId: 6,
      sets: [
        { w: 90, r: 8 },
        { w: 100, r: 8, pr: true },
      ],
    },
  ]);

  add(daysAgo(9), null, [
    {
      exerciseId: 1,
      sets: [
        { w: 80, r: 5 },
        { w: 90, r: 5 },
        { w: 100, r: 4 },
      ],
    },
    {
      exerciseId: 2,
      sets: [
        { w: 30, r: 8 },
        { w: 32.5, r: 6, pr: true },
      ],
    },
  ]);

  add(daysAgo(11), null, [
    {
      exerciseId: 3,
      sets: [
        { w: 70, r: 8 },
        { w: 80, r: 8 },
        { w: 82.5, r: 6 },
      ],
    },
    {
      exerciseId: 4,
      sets: [
        { w: 0, r: 10 },
        { w: 0, r: 8 },
        { w: 5, r: 6 },
      ],
    },
  ]);

  add(daysAgo(13), "Leg day", [
    {
      exerciseId: 5,
      sets: [
        { w: 100, r: 5 },
        { w: 120, r: 5 },
        { w: 130, r: 3 },
      ],
    },
  ]);

  add(daysAgo(16), null, [
    {
      exerciseId: 1,
      sets: [
        { w: 80, r: 5 },
        { w: 85, r: 5 },
        { w: 95, r: 5 },
      ],
    },
    {
      exerciseId: 7,
      sets: [
        { w: 50, r: 6 },
        { w: 55, r: 4 },
      ],
    },
  ]);

  add(daysAgo(23), null, [
    {
      exerciseId: 3,
      sets: [
        { w: 70, r: 8 },
        { w: 75, r: 8 },
      ],
    },
  ]);

  add(daysAgo(30), null, [
    {
      exerciseId: 5,
      sets: [
        { w: 100, r: 5 },
        { w: 115, r: 5 },
      ],
    },
    {
      exerciseId: 6,
      sets: [
        { w: 85, r: 8 },
        { w: 95, r: 8 },
      ],
    },
  ]);

  add(daysAgo(37), null, [
    {
      exerciseId: 1,
      sets: [
        { w: 75, r: 5 },
        { w: 85, r: 5 },
        { w: 92.5, r: 5 },
      ],
    },
  ]);

  add(daysAgo(44), null, [
    {
      exerciseId: 3,
      sets: [
        { w: 65, r: 8 },
        { w: 72.5, r: 8 },
      ],
    },
    {
      exerciseId: 4,
      sets: [
        { w: 0, r: 8 },
        { w: 0, r: 6 },
      ],
    },
  ]);

  return out;
})();

// --- Body metrics fixtures ---

const METRICS = [
  { id: 1, name: "Weight", unit: "kg", is_derived: false },
  { id: 2, name: "Neck", unit: "cm", is_derived: false },
  { id: 3, name: "Waist", unit: "cm", is_derived: false },
  { id: 4, name: "Hip", unit: "cm", is_derived: false },
  { id: 5, name: "Upper Middle Arm (Right)", unit: "cm", is_derived: false },
  { id: 6, name: "BMI", unit: "kg/m²", is_derived: true },
  { id: 7, name: "Body Fat (Navy)", unit: "%", is_derived: true },
  { id: 8, name: "FFMI (Navy)", unit: "kg/m²", is_derived: true },
];

type RawMeasurement = { date: string; metricId: number; value: number };
type StoredMeasurement = RawMeasurement & { id: number };

const BODY_MEASUREMENTS: StoredMeasurement[] = (() => {
  const raws: RawMeasurement[] = [];
  const entries: [number, { w: number; neck: number; waist: number }][] = [
    [49, { w: 82.4, neck: 39.5, waist: 86.0 }],
    [42, { w: 82.1, neck: 39.5, waist: 85.5 }],
    [35, { w: 81.6, neck: 39.6, waist: 85.0 }],
    [28, { w: 81.3, neck: 39.6, waist: 84.6 }],
    [21, { w: 80.9, neck: 39.7, waist: 84.1 }],
    [14, { w: 80.4, neck: 39.8, waist: 83.5 }],
    [7, { w: 79.8, neck: 39.9, waist: 82.8 }],
  ];
  for (const [ago, m] of entries) {
    const date = daysAgo(ago);
    raws.push({ date, metricId: 1, value: m.w });
    raws.push({ date, metricId: 2, value: m.neck });
    raws.push({ date, metricId: 3, value: m.waist });
  }
  return raws.map((r, i) => ({ ...r, id: i + 1 }));
})();

// Derived metric computations (mirror Rust logic, simplified for male)
const HEIGHT_CM = 182;

function calcBmi(kg: number) {
  const m = HEIGHT_CM / 100;
  return kg / (m * m);
}
function calcBfMale(waist: number, neck: number) {
  return (
    495 /
      (1.0324 -
        0.19077 * Math.log10(waist - neck) +
        0.15456 * Math.log10(HEIGHT_CM)) -
    450
  );
}
function calcFfmi(bfPct: number, kg: number) {
  const ffm = kg * (1 - bfPct / 100);
  const m = HEIGHT_CM / 100;
  return ffm / (m * m);
}

function buildMeasurementsForDate(date: string, useLatestOnOrBefore: boolean) {
  // Collect latest per metric on or before date, or exact date
  const perMetric = new Map<
    number,
    { value: number; date: string; id: number }
  >();
  for (const m of BODY_MEASUREMENTS) {
    const match = useLatestOnOrBefore ? m.date <= date : m.date === date;
    if (!match) continue;
    const existing = perMetric.get(m.metricId);
    if (!existing || existing.date < m.date) {
      perMetric.set(m.metricId, { value: m.value, date: m.date, id: m.id });
    }
  }

  const out: Array<{
    metric: { name: string; unit: string; id: number; is_derived: boolean };
    value: number;
    date: string | null;
    id: number | null;
  }> = [];

  for (const [metricId, entry] of perMetric) {
    const meta = METRICS.find((x) => x.id === metricId)!;
    out.push({
      metric: {
        name: meta.name,
        unit: meta.unit,
        id: meta.id,
        is_derived: false,
      },
      value: entry.value,
      date: entry.date,
      id: entry.id,
    });
  }

  // Derived values
  const weight = perMetric.get(1)?.value;
  const neck = perMetric.get(2)?.value;
  const waist = perMetric.get(3)?.value;
  const latestDate =
    [...perMetric.values()]
      .map((e) => e.date)
      .sort()
      .pop() ?? null;

  if (weight !== undefined) {
    out.push({
      metric: { name: "BMI", unit: "kg/m²", id: 5, is_derived: true },
      value: calcBmi(weight),
      date: latestDate,
      id: null,
    });
  }
  if (waist !== undefined && neck !== undefined) {
    const bf = calcBfMale(waist, neck);
    out.push({
      metric: {
        name: "Body Fat (Navy)",
        unit: "%",
        id: 6,
        is_derived: true,
      },
      value: bf,
      date: latestDate,
      id: null,
    });
    if (weight !== undefined) {
      out.push({
        metric: {
          name: "FFMI (Navy)",
          unit: "kg/m²",
          id: 7,
          is_derived: true,
        },
        value: calcFfmi(bf, weight),
        date: latestDate,
        id: null,
      });
    }
  }
  return out;
}

// --- Exercise history / graph helpers ---

function expandExerciseWithSets(we: MockWorkoutExercise) {
  const ex = EXERCISES.find((e) => e.id === we.exercise_id)!;
  return {
    exercise: { id: ex.id, name: ex.name },
    category: categoryName(ex.category_id),
    workout_exercise_id: we.workout_exercise_id,
    exercise_order: we.exercise_order,
    sets: we.sets,
  };
}

function estimate1rm(weight: number, reps: number): number {
  if (reps <= 0) return 0;
  if (reps <= 10) return weight * (36 / (37 - reps));
  return weight * (1 + reps / 30);
}

// --- Dispatcher ---

export function mockInvoke<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  const a = (args ?? {}) as Record<string, any>;
  const result = dispatch(cmd, a);
  return Promise.resolve(result as T);
}

function dispatch(cmd: string, a: Record<string, any>): unknown {
  switch (cmd) {
    case "get_settings":
      return {
        height: HEIGHT_CM,
        unit: "Kg",
        dark_mode: false,
        sex: "Male",
      };

    case "list_exercise_categories":
      return CATEGORIES.map((c) => ({ id: c.id, name: c.name }));

    case "list_exercises_in_category": {
      const cid = a.categoryId ?? a.category_id;
      return EXERCISES.filter((e) => e.category_id === cid)
        .map((e) => ({ id: e.id, name: e.name }))
        .sort((x, y) => x.name.localeCompare(y.name));
    }

    case "list_metrics":
      return METRICS.map((m) => ({ ...m }));

    case "list_templates":
      return [
        { id: 1, name: "Push A" },
        { id: 2, name: "Pull A" },
        { id: 3, name: "Legs A" },
      ];

    case "get_exercise": {
      const id = a.id;
      const ex = EXERCISES.find((e) => e.id === id);
      if (!ex) throw new Error(`Exercise ${id} not found`);
      return { id: ex.id, name: ex.name };
    }

    case "get_active_dates": {
      const y: number = a.year;
      const m: number = a.month;
      const prefix = `${y}-${String(m).padStart(2, "0")}`;
      return [
        ...new Set(
          WORKOUTS.filter((w) => w.date.startsWith(prefix)).map((w) => w.date),
        ),
      ].sort();
    }

    case "get_workout_for_date": {
      const date: string = a.date;
      const wk = WORKOUTS.find((w) => w.date === date);
      if (!wk) return [];
      return wk.exercises.map(expandExerciseWithSets);
    }

    case "get_workout_id_for_date": {
      const date: string = a.date;
      return WORKOUTS.find((w) => w.date === date)?.id ?? null;
    }

    case "get_workout_name_for_date": {
      const date: string = a.date;
      return WORKOUTS.find((w) => w.date === date)?.name ?? null;
    }

    case "get_workout_exercise_context": {
      const weId = a.workoutExerciseId ?? a.workout_exercise_id;
      for (const w of WORKOUTS) {
        const we = w.exercises.find((x) => x.workout_exercise_id === weId);
        if (we) {
          const ex = EXERCISES.find((e) => e.id === we.exercise_id)!;
          return { exercise_name: ex.name, date: w.date };
        }
      }
      throw new Error(`workout_exercise ${weId} not found`);
    }

    case "get_sets_for_workout_exercise": {
      const weId = a.workoutExerciseId ?? a.workout_exercise_id;
      for (const w of WORKOUTS) {
        const we = w.exercises.find((x) => x.workout_exercise_id === weId);
        if (we) return we.sets;
      }
      return [];
    }

    case "get_workouts_for_range": {
      const from: string = a.fromDate ?? a.from_date;
      const to: string = a.toDate ?? a.to_date;
      return WORKOUTS.filter((w) => w.date >= from && w.date <= to)
        .sort((x, y) => y.date.localeCompare(x.date))
        .map((w) => ({
          date: w.date,
          exercises: w.exercises.map(expandExerciseWithSets),
        }));
    }

    case "get_exercise_history": {
      const exerciseId = a.exerciseId ?? a.exercise_id;
      const days: Array<{ date: string; exercises: any[] }> = [];
      for (const w of [...WORKOUTS].sort((x, y) =>
        y.date.localeCompare(x.date),
      )) {
        const matching = w.exercises.filter(
          (we) => we.exercise_id === exerciseId,
        );
        if (matching.length) {
          days.push({
            date: w.date,
            exercises: matching.map(expandExerciseWithSets),
          });
        }
      }
      return days;
    }

    case "get_exercise_graph_data": {
      const exerciseId = a.exerciseId ?? a.exercise_id;
      const from: string = a.fromDate ?? a.from_date;
      const to: string = a.toDate ?? a.to_date;
      const dayMax = new Map<string, number>();
      for (const w of WORKOUTS) {
        if (w.date < from || w.date > to) continue;
        for (const we of w.exercises) {
          if (we.exercise_id !== exerciseId) continue;
          for (const s of we.sets) {
            const rm = estimate1rm(s.weight_kg, s.reps);
            if (rm > (dayMax.get(w.date) ?? 0)) dayMax.set(w.date, rm);
          }
        }
      }
      return [...dayMax.entries()]
        .map(([date, value]) => ({ date, value }))
        .sort((x, y) => x.date.localeCompare(y.date));
    }

    case "get_rep_maxes": {
      const exerciseId = a.exerciseId ?? a.exercise_id;
      const rows: Array<{ date: string; reps: number; weight_kg: number }> = [];
      for (const w of WORKOUTS) {
        for (const we of w.exercises) {
          if (we.exercise_id !== exerciseId) continue;
          for (const s of we.sets) {
            if (s.is_current_pr) {
              rows.push({
                date: w.date,
                reps: s.reps,
                weight_kg: s.weight_kg,
              });
            }
          }
        }
      }
      return rows.sort((x, y) => y.weight_kg - x.weight_kg);
    }

    case "get_last_set": {
      const exerciseId = a.exerciseId ?? a.exercise_id;
      const sorted = [...WORKOUTS].sort((x, y) => y.date.localeCompare(x.date));
      for (const w of sorted) {
        for (const we of w.exercises) {
          if (we.exercise_id !== exerciseId) continue;
          const last = we.sets[we.sets.length - 1];
          if (last) return { weight: last.weight_kg, reps: last.reps };
        }
      }
      return null;
    }

    case "get_measurements_for_date":
      return buildMeasurementsForDate(a.date, false);

    case "get_last_measurements_for_date":
      return buildMeasurementsForDate(a.date, true);

    case "get_measurement_history": {
      const byDate = new Map<string, any[]>();
      for (const m of BODY_MEASUREMENTS) {
        const meta = METRICS.find((x) => x.id === m.metricId)!;
        if (!byDate.has(m.date)) byDate.set(m.date, []);
        byDate.get(m.date)!.push({
          metric: {
            name: meta.name,
            unit: meta.unit,
            id: meta.id,
            is_derived: false,
          },
          value: m.value,
          date: m.date,
          id: m.id,
        });
      }
      const days = [...byDate.entries()]
        .map(([date, measurements]) => ({ date, measurements }))
        .sort((x, y) => y.date.localeCompare(x.date));
      for (const day of days) {
        const w = day.measurements.find(
          (x: any) => x.metric.name === "Weight",
        )?.value;
        const n = day.measurements.find(
          (x: any) => x.metric.name === "Neck",
        )?.value;
        const wa = day.measurements.find(
          (x: any) => x.metric.name === "Waist",
        )?.value;
        if (w !== undefined) {
          day.measurements.push({
            metric: { name: "BMI", unit: "kg/m²", id: 5, is_derived: true },
            value: calcBmi(w),
            date: day.date,
            id: null,
          });
        }
        if (w !== undefined && n !== undefined && wa !== undefined) {
          const bf = calcBfMale(wa, n);
          day.measurements.push({
            metric: {
              name: "Body Fat (Navy)",
              unit: "%",
              id: 6,
              is_derived: true,
            },
            value: bf,
            date: day.date,
            id: null,
          });
          day.measurements.push({
            metric: {
              name: "FFMI (Navy)",
              unit: "kg/m²",
              id: 7,
              is_derived: true,
            },
            value: calcFfmi(bf, w),
            date: day.date,
            id: null,
          });
        }
      }
      return days;
    }

    default:
      // Mutations and anything else: no-op. Return null so callers that
      // discard the result don't crash; routes that depend on a specific
      // non-null shape will need a case added above.
      if (import.meta.env?.DEV) {
        console.warn(`[mockInvoke] unhandled command: ${cmd}`);
      }
      return null;
  }
}
