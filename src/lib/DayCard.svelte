<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { dndzone } from "svelte-dnd-action";
    import { formatDate } from "$lib/date";
    import type { ExerciseWithSets } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";
    import type { Measurement } from "$lib/body";

    let { date }: { date: string } = $props();

    let exercises = $state<ExerciseWithSets[]>([]);

    async function loadExercises() {
        const result = await invoke<ExerciseWithSets[]>(
            "get_workout_for_date",
            { date },
        );
        exercises = result.map((e) => ({ ...e, id: e.exercise_id }));
    }

    onMount(loadExercises);

    function handleConsider(e: CustomEvent) {
        exercises = e.detail.items;
    }

    function handleFinalize(e: CustomEvent) {
        exercises = e.detail.items;
        invoke("reorder_exercises", {
            date,
            orderedExerciseIds: exercises.map((ex) => ex.exercise_id),
        });
    }
</script>

<article class="day-card" id="day-{date}">
    <div class="day-card-header">
        <span class="day-label">{formatDate(date)}</span>
        <div class="day-card-btns">
            <button
                class="measure-btn-inline"
                onclick={() => goto(`/body/${date}`)}>+ Body</button
            >
            <button
                class="add-btn-inline"
                onclick={() => goto(`/exercises/${date}`)}>+ Add</button
            >
        </div>
    </div>

    {#if exercises.length === 0}
        <p class="empty">Rest day</p>
    {:else}
        <div
            class="list"
            use:dndzone={{ items: exercises, flipDurationMs: 150 }}
            onconsider={handleConsider}
            onfinalize={handleFinalize}
        >
            {#each exercises as ex (ex.id)}
                <div class="exercise-card">
                    <button
                        class="exercise-card-header"
                        onclick={() =>
                            goto(`/exercise/${ex.exercise_id}/${date}`)}
                    >
                        <span class="drag-handle">≡</span>
                        <span>{ex.exercise_name}</span>
                        <span class="muted">→</span>
                    </button>
                    <div class="exercise-card-sets">
                        {#each ex.sets as set, i}
                            <div class="set-row">
                                <span class="set-label">{i + 1}</span>
                                <span class="set-stats">
                                    <span class="stat-val stat-val--weight"
                                        >{formatWeight(set.weight_kg)}</span
                                    ><span class="stat-unit">kg</span>
                                    <span class="stat-val stat-val--reps"
                                        >{set.reps}</span
                                    ><span class="stat-unit">reps</span>
                                </span>
                                <span class="set-badge">
                                    {#if set.is_current_pr}
                                        <span class="pr-badge pr-badge--current"
                                            >PR</span
                                        >
                                    {:else if set.was_pr_at_time}
                                        <span
                                            class="pr-badge pr-badge--historic"
                                            >PR</span
                                        >
                                    {/if}
                                </span>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</article>
