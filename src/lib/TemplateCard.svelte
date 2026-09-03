<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "$lib/tauri";
    import { selectionFeedback } from "@tauri-apps/plugin-haptics";

    import { onMount } from "svelte";
    import { dndzone, type DndEvent } from "svelte-dnd-action";
    import {
        GripVertical,
        ChevronRight,
        Dumbbell,
        X,
        Trash,
        Trash2,
        Merge,
        Pencil,
    } from "lucide-svelte";
    import type {
        ExerciseWithSets,
        NamedId,
        TemplateWithExercises,
    } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";

    let { template, listTemplates, onLoaded } = $props<{
        template: NamedId;
        listTemplates: () => Promise<void>;
        onLoaded?: (id: number) => void;
    }>();

    let nameId: NamedId = template;
    const date = new URLSearchParams(location.search).get("date");
    let workoutTitle = $state("Template");
    let exercises = $state<ExerciseWithSets[]>([]);

    let confirmingDelete = $state(false);
    let enteringName = $state(false);

    let dragDisabled = $state(true);

    async function loadTemplate() {
        const result = await invoke<TemplateWithExercises>("get_template", {
            id: nameId.id,
        });
        exercises = result.exercises.map((e) => ({
            ...e,
            id: e.workout_exercise_id,
        }));
        workoutTitle = result.template.name;
    }

    async function deleteTemplate() {
        await invoke("delete_template", {
            templateId: nameId.id,
        });
        await listTemplates();
    }

    async function renameTemplate() {
        await invoke("rename_template", {
            id: nameId.id,
            name: workoutTitle,
        });
        enteringName = false;
        await listTemplates();
    }

    onMount(async () => {
        await loadTemplate();
        onLoaded?.(nameId.id);
    });

    const handleConsider = (
        evt: CustomEvent<DndEvent<ExerciseWithSets>>,
    ) => {
        exercises = evt.detail.items;
    };
    const handleFinalize = (
        evt: CustomEvent<DndEvent<ExerciseWithSets>>,
    ) => {
        exercises = evt.detail.items;
        invoke("reorder_template_exercises", {
            orderedTemplateExerciseIds: exercises.map(
                (ex) => ex.workout_exercise_id,
            ),
        });
        dragDisabled = true;
        loadTemplate();
    };
    const startDrag = () => {
        dragDisabled = false;
    };

    let touchStartX = $state(0);
    let touchStartY = $state(0);
    let longPressJustFired = $state(false);
    function handlePointerDown(e: PointerEvent, we_id: number) {
        longPressJustFired = false;
        touchStartX = e.clientX;
        touchStartY = e.clientY;
        if (!dragDisabled) return;
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
        for (const te_id of selectedExercises) {
            await invoke("remove_exercise_from_template", {
                templateExerciseId: te_id,
            });
        }
        selectedExercises = [];
        selectMode = false;
        loadTemplate();
    }

    async function mergeSelectedExercises() {
        let exerciseIds = selectedExercises.map(
            (we_id) =>
                exercises.find((e) => e.workout_exercise_id === we_id)?.exercise
                    .id,
        );
        // Can only merge exercises with the same exercise_id
        const uniqueExerciseIds = new Set(exerciseIds);
        if (uniqueExerciseIds.size !== 1) {
            return;
        }

        await invoke("merge_template_exercises", {
            templateExerciseIds: selectedExercises,
        });
        selectedExercises = [];
        selectMode = false;
        loadTemplate();
    }
</script>

<article class="day-card" id="template-{nameId.id}">
    <div class="day-card-header">
        <span class="day-label"></span>
        <div class="day-card-btns"></div>
    </div>

    <div class="workout-card">
        <div class="workout-card-header">
            {#if enteringName}
                <input
                    class="workout-title-input"
                    bind:value={workoutTitle}
                    onblur={() => renameTemplate()}
                />
            {:else}
                <div style="display: flex; gap: 10px;">
                    <h2 class="workout-title">{workoutTitle}</h2>
                    <button
                        class="back-btn"
                        onclick={() => (enteringName = true)}
                    >
                        <Pencil size={18} strokeWidth={1.5} />
                    </button>
                </div>
            {/if}
            <div class="workout-card-btns">
                <button
                    class="back-btn"
                    onclick={() => (confirmingDelete = true)}
                >
                    <Trash2 size={18} strokeWidth={1.5} />
                </button>
                <button
                    class="back-btn"
                    onclick={() =>
                        goto(`/exercises/${date}?fromTemplate=${nameId.id}`)}
                >
                    <Dumbbell size={18} strokeWidth={1.5} />
                </button>
            </div>
        </div>
        {#if confirmingDelete}
            <span style="font-size: 0.9rem;"
                >Delete template? This cannot be undone.</span
            >
            <div style="display: flex; gap: 0.5rem;">
                <button
                    class="delete-btn"
                    style="flex:1;"
                    onclick={deleteTemplate}
                >
                    Delete template
                </button>
                <button
                    class="update-btn"
                    style="flex:1;"
                    onclick={() => (confirmingDelete = false)}
                >
                    Cancel
                </button>
            </div>
        {/if}
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
                <button
                    class="exercise-card"
                    class:selected={selectedExercises.includes(
                        ex.workout_exercise_id,
                    )}
                    onpointerdown={(e) =>
                        handlePointerDown(e, ex.workout_exercise_id)}
                    onpointermove={handlePointerMove}
                    onpointerup={stopHold}
                    onpointercancel={stopHold}
                    onclick={() =>
                        selectMode
                            ? selectExercise(ex.workout_exercise_id)
                            : goto(
                                  `/exercise/${ex.exercise.id}/${ex.workout_exercise_id}?fromDate=${date}&fromTemplate=${nameId.id}`,
                              )}
                >
                    <div class="exercise-card-header">
                        <span
                            class="drag-handle"
                            role="button"
                            tabindex="0"
                            aria-label="Drag to reorder"
                            onpointerdown={startDrag}
                            ><GripVertical size={16} strokeWidth={1.5} /></span
                        >
                        <span>{ex.exercise.name}</span>
                        <span class="muted"
                            ><ChevronRight size={16} strokeWidth={1.5} /></span
                        >
                    </div>
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
                </button>
            {/each}
        </div>
    </div>
    {#if selectMode}
        <div class="selection-bar">
            <button class="icon-btn" onclick={() => cancelSelection()}>
                <X size={16} strokeWidth={1.5} />
            </button>
            <span>{selectedExercises.length} selected</span>
            <button class="icon-btn" onclick={() => deleteSelectedExercises()}>
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
</article>
