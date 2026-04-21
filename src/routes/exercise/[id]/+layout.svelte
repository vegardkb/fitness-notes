<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "$lib/tauri";
    import ExerciseHeader from "$lib/ExerciseHeader.svelte";
    import type { WorkoutExerciseContext } from "$lib/exercise";
    import { exerciseHrefs } from "$lib/exercise";

    let { children } = $props();

    let exerciseId = $derived(Number(page.params.id));
    let workoutExerciseContext: WorkoutExerciseContext = $state({
        exercise_name: "",
        date: "",
    });

    let workoutExerciseId = $state(0);

    $effect(() => {
        if (page.params.we_id) {
            workoutExerciseId = Number(page.params.we_id);
        } else {
            invoke<number>("get_last_workout_exercise", {
                exerciseId: exerciseId,
            }).then((id) => {
                workoutExerciseId = id;
            });
        }
    });

    const activeTab = $derived(
        page.route.id?.endsWith("/history")
            ? "history"
            : page.route.id?.endsWith("/graph")
              ? "graph"
              : page.route.id?.endsWith("/prs")
                ? "prs"
                : "sets",
    );

    const hrefs = $derived(
        exerciseHrefs(
            exerciseId,
            workoutExerciseId,
            workoutExerciseContext.date,
        ),
    );

    let exerciseName: string = $derived(workoutExerciseContext.exercise_name);
    let date: string = $derived(workoutExerciseContext.date);

    $effect(() => {
        console.log(workoutExerciseId);
        invoke<WorkoutExerciseContext>("get_workout_exercise_context", {
            workoutExerciseId,
        }).then((context) => {
            workoutExerciseContext = context;
            console.log(workoutExerciseContext);
        });
    });
</script>

<div class="page">
    <ExerciseHeader
        feedHref={hrefs.feedHref}
        setsHref={hrefs.setsHref}
        historyHref={hrefs.historyHref}
        graphHref={hrefs.graphHref}
        prsHref={hrefs.prsHref}
        {exerciseName}
        {activeTab}
        {date}
    />
    {@render children()}
</div>
