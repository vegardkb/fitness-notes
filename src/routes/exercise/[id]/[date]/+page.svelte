<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { dndzone } from "svelte-dnd-action";

    type Set = {
        id: number;
        set_order: number;
        weight_kg: number;
        reps: number;
        notes: string | null;
        was_pr_at_time: boolean;
        is_current_pr: boolean;
    };

    type ExerciseWithSets = {
        exercise_id: number;
        exercise_name: string;
        sets: Set[];
    };

    type Exercise = {
        id: number;
        name: string;
    };

    const exerciseId = $derived(Number(page.params.id ?? "0"));
    const date = $derived(page.params.date ?? "");

    let exerciseName = $state("");
    let sets = $state<Set[]>([]);
    let weightInput = $state("");
    let repsInput = $state("");
    let adding = $state(false);
    let set_selected = $state<Set | null>(null);

    function defaultToLastSet(setList: Set[]) {
        const last = setList.at(-1);
        if (last) {
            weightInput = String(last.weight_kg);
            repsInput = String(last.reps);
        }
    }

    function selectSet(set: Set) {
        if (set_selected?.id === set.id) {
            set_selected = null;
            defaultToLastSet(sets);
        } else {
            set_selected = set;
            weightInput = String(set.weight_kg);
            repsInput = String(set.reps);
        }
    }

    function formatWeight(kg: number): string {
        const f2 = kg.toFixed(2);
        return f2.endsWith("0") ? kg.toFixed(1) : f2;
    }

    async function refreshSets() {
        const workout = await invoke<ExerciseWithSets[]>("get_workout_for_date", { date });
        sets = workout.find((e) => e.exercise_id === exerciseId)?.sets ?? [];
    }

    onMount(async () => {
        const [workout, exercise] = await Promise.all([
            invoke<ExerciseWithSets[]>("get_workout_for_date", { date }),
            invoke<Exercise>("get_exercise", { id: exerciseId }),
        ]);
        exerciseName = exercise.name;
        sets = workout.find((e) => e.exercise_id === exerciseId)?.sets ?? [];
        defaultToLastSet(sets);
    });

    async function addSet() {
        if (!weightInput || !repsInput) return;
        adding = true;
        try {
            await invoke<Set>("upsert_set", {
                id: set_selected?.id ?? null,
                date,
                exerciseId,
                weightKg: parseFloat(weightInput),
                reps: parseInt(repsInput),
                notes: null,
            });
            set_selected = null;
            await refreshSets();
            defaultToLastSet(sets);
        } finally {
            adding = false;
        }
    }

    async function deleteSet() {
        if (!set_selected) return;
        await invoke("delete_set", { id: set_selected.id });
        set_selected = null;
        await refreshSets();
        defaultToLastSet(sets);
    }

    function handleSetConsider(e: CustomEvent) {
        sets = e.detail.items;
    }

    function handleSetFinalize(e: CustomEvent) {
        sets = e.detail.items;
        invoke("reorder_sets", {
            date,
            exerciseId,
            orderedSetIds: sets.map((s) => s.id),
        }).then(() => refreshSets());
    }
</script>

<div class="page">
    <div class="header">
        <a class="back-btn" href="/day/{date}">←</a>
        <h1>{exerciseName}</h1>
    </div>

    {#if sets.length === 0}
        <p class="empty">No sets yet. Add your first set below.</p>
    {:else}
        <div
            class="list"
            use:dndzone={{ items: sets, flipDurationMs: 150 }}
            onconsider={handleSetConsider}
            onfinalize={handleSetFinalize}
        >
            {#each sets as set (set.id)}
                <button
                    class="list-item"
                    class:list-item--selected={set_selected?.id === set.id}
                    onclick={() => selectSet(set)}
                >
                    <span class="drag-handle">≡</span>
                    <span class="set-stats">
                        <span class="stat-val stat-val--weight">{formatWeight(set.weight_kg)}</span><span class="stat-unit">kg</span>
                        <span class="stat-val stat-val--reps">{set.reps}</span><span class="stat-unit">reps</span>
                    </span>
                    <span class="set-badge">
                        {#if set.is_current_pr}
                            <span class="pr-badge pr-badge--current">PR</span>
                        {:else if set.was_pr_at_time}
                            <span class="pr-badge pr-badge--historic">PR</span>
                        {/if}
                    </span>
                </button>
            {/each}
        </div>
    {/if}

    <div class="form">
        <div class="form-row">
            <div class="field">
                <label for="weight">Weight (kg)</label>
                <input
                    id="weight"
                    type="number"
                    step="0.5"
                    min="0"
                    placeholder="100"
                    bind:value={weightInput}
                />
            </div>
            <div class="field">
                <label for="reps">Reps</label>
                <input
                    id="reps"
                    type="number"
                    min="1"
                    placeholder="5"
                    bind:value={repsInput}
                />
            </div>
        </div>
        {#if set_selected}
            <button
                class="update-btn"
                onclick={addSet}
                disabled={adding || !weightInput || !repsInput}
            >
                {adding ? "Updating…" : "Update set"}
            </button>
            <button class="delete-btn" onclick={deleteSet}>Delete</button>
        {:else}
            <button
                class="add-btn"
                onclick={addSet}
                disabled={adding || !weightInput || !repsInput}
            >
                {adding ? "Adding…" : "Add set"}
            </button>
        {/if}
    </div>
</div>
