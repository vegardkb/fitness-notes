<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { selectionFeedback } from "@tauri-apps/plugin-haptics";

    import { onMount } from "svelte";
    import { dndzone } from "svelte-dnd-action";
    import { formatDate } from "$lib/date";
    import {
        GripVertical,
        ChevronRight,
        PersonStanding,
        Dumbbell,
        X,
        Trash,
        Merge,
    } from "lucide-svelte";
    import type { ExerciseWithSets } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";

    let { date }: { date: string } = $props();

    let exercises = $state<ExerciseWithSets[]>([]);

    let dragDisabled = $state(true);

    async function loadExercises() {
        const result = await invoke<ExerciseWithSets[]>(
            "get_workout_for_date",
            { date },
        );
        exercises = result.map((e) => ({ ...e, id: e.workout_exercise_id }));
    }

    onMount(loadExercises);

    const handleConsider = (evt) => {
        exercises = evt.detail.items;
    };
    const handleFinalize = (evt) => {
        exercises = evt.detail.items;
        invoke("reorder_exercises", {
            orderedWorkoutExerciseIds: exercises.map(
                (ex) => ex.workout_exercise_id,
            ),
        });
        dragDisabled = true;
        loadExercises();
    };
    const startDrag = () => {
        dragDisabled = false;
    };
    const stopDrag = () => {
        dragDisabled = true;
    };

    let touchStartX = $state(0);
    let touchStartY = $state(0);
    let longPressJustFired = $state(false);
    function handlePointerDown(e: PointerEvent, we_id: number) {
        longPressJustFired = false;
        touchStartX = e.clientX;
        touchStartY = e.clientY;
        startHold(we_id);
    }

    function handlePointerMove(e: PointerEvent) {
        const dx = Math.abs(e.clientX - touchStartX);
        const dy = Math.abs(e.clientY - touchStartY);
        if (dx > 10 || dy > 10) {
            stopHold();
        }
    }

    let timer = $state<number | null>(null);
    async function startHold(we_id: number) {
        timer = setTimeout(async () => {
            await selectionFeedback();
            selectExercise(we_id);
            longPressJustFired = true;
            timer = null;
        }, 400);
    }

    function stopHold() {
        if (timer !== null) {
            clearTimeout(timer);
            timer = null;
        }
    }

    let selectMode = $state(false);
    let selectedExercises = $state<number[]>([]);
    function selectExercise(we_id: number) {
        if (longPressJustFired) {
            longPressJustFired = false;
            return;
        }
        selectMode = true;
        if (selectedExercises.includes(we_id)) {
            selectedExercises = selectedExercises.filter((id) => id !== we_id);
            if (selectedExercises.length === 0) {
                selectMode = false;
            }
        } else {
            selectedExercises.push(we_id);
        }
    }

    function cancelSelection() {
        selectedExercises = [];
        selectMode = false;
    }

    async function deleteSelectedExercises() {
        for (const we_id of selectedExercises) {
            await invoke("remove_exercise_from_workout", {
                workoutExerciseId: we_id,
            });
        }
        selectedExercises = [];
        selectMode = false;
        loadExercises();
    }

    async function mergeSelectedExercises() {
        let exerciseIds = selectedExercises.map(
            (we_id) =>
                exercises.find((e) => e.workout_exercise_id === we_id)
                    ?.exercise_id,
        );
        // Can only merge exercises with the same exercise_id
        const uniqueExerciseIds = new Set(exerciseIds);
        if (uniqueExerciseIds.size !== 1) {
            return;
        }

        await invoke("merge_workout_exercises", {
            workoutExerciseIds: selectedExercises,
        });
        selectedExercises = [];
        selectMode = false;
        loadExercises();
    }
</script>

<article class="day-card" id="day-{date}">
    <div class="day-card-header">
        <span class="day-label">{formatDate(date)}</span>
        <div class="day-card-btns">
            <button class="back-btn" onclick={() => goto(`/body/${date}`)}>
                <PersonStanding size={18} strokeWidth={2} />
            </button>
            <button class="back-btn" onclick={() => goto(`/exercises/${date}`)}>
                <Dumbbell size={18} strokeWidth={1.5} />
            </button>
        </div>
    </div>

    {#if exercises.length === 0}
        <p class="empty">Rest day</p>
    {:else}
        <div
            class="list"
            use:dndzone={{
                items: exercises,
                flipDurationMs: 150,
                dragDisabled,
            }}
            onconsider={handleConsider}
            onfinalize={handleFinalize}
        >
            {#each exercises as ex (ex.id)}
                <div
                    class="exercise-card"
                    class:selected={selectedExercises.includes(
                        ex.workout_exercise_id,
                    )}
                >
                    <button
                        class="exercise-card-header"
                        onpointerdown={(e) =>
                            handlePointerDown(e, ex.workout_exercise_id)}
                        onpointermove={handlePointerMove}
                        onpointerup={stopHold}
                        onpointercancel={stopHold}
                        onclick={() =>
                            selectMode
                                ? selectExercise(ex.workout_exercise_id)
                                : goto(
                                      `/exercise/${ex.exercise_id}/${ex.workout_exercise_id}`,
                                  )}
                    >
                        <span
                            class="drag-handle"
                            role="button"
                            tabindex="0"
                            aria-label="Drag to reorder"
                            onpointerdown={startDrag}
                            ><GripVertical size={16} strokeWidth={1.5} /></span
                        >
                        <span>{ex.exercise_name}</span>
                        <span class="muted"
                            ><ChevronRight size={16} strokeWidth={1.5} /></span
                        >
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
        {#if selectMode}
            <div class="selection-bar">
                <button class="icon-btn" onclick={() => cancelSelection()}>
                    <X size={16} strokeWidth={1.5} />
                </button>
                <span>{selectedExercises.length} selected</span>
                <button
                    class="icon-btn"
                    onclick={() => deleteSelectedExercises()}
                >
                    <Trash size={16} strokeWidth={1.5} />
                </button>
                <button
                    class="icon-btn"
                    onclick={() => mergeSelectedExercises()}
                    disabled={selectedExercises.length < 2}
                >
                    <Merge size={16} strokeWidth={1.5} />
                </button>
            </div>
        {/if}
    {/if}
</article>
