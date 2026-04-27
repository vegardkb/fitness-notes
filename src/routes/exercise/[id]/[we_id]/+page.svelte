<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "$lib/tauri";
    import { onMount } from "svelte";
    import { dndzone } from "svelte-dnd-action";
    import type {
        Set,
        WorkoutExerciseContext,
        SetMinimal,
    } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";
    import { SquareMinus, SquarePlus, GripVertical } from "lucide-svelte";

    const workoutExerciseId = $derived(Number(page.params.we_id ?? "0"));
    const exerciseId = $derived(Number(page.params.id));

    let sets = $state<Set[]>([]);
    let date = $state<string>("");
    let weightInput: number = $state(NaN);
    let repsInput: number = $state(NaN);
    let adding = $state(false);
    let set_selected = $state<Set | null>(null);
    let lastSet: SetMinimal = $state({ weight: 0, reps: 0 });

    function defaultToLastSet() {
        if (lastSet) {
            weightInput = lastSet.weight;
            repsInput = lastSet.reps;
        }
    }

    function selectSet(set: Set) {
        if (set_selected?.id === set.id) {
            set_selected = null;
            defaultToLastSet();
        } else {
            set_selected = set;
            lastSet = { weight: weightInput, reps: repsInput };
            weightInput = set.weight_kg;
            repsInput = set.reps;
        }
    }

    async function refreshSets() {
        const setsData = await invoke<Set[]>("get_sets_for_workout_exercise", {
            workoutExerciseId: workoutExerciseId,
        });
        sets = setsData;
    }

    onMount(async () => {
        console.log(date);
        const setsData = await invoke<Set[]>("get_sets_for_workout_exercise", {
            workoutExerciseId: workoutExerciseId,
        });
        const lastSetData = await invoke<SetMinimal | null>("get_last_set", {
            exerciseId: exerciseId,
        });
        const weContext = await invoke<WorkoutExerciseContext>(
            "get_workout_exercise_context",
            {
                workoutExerciseId: workoutExerciseId,
            },
        );

        sets = setsData;
        if (lastSetData) lastSet = lastSetData;
        date = weContext.date;
        defaultToLastSet();
    });

    async function addSet() {
        adding = true;
        try {
            await invoke<Set>("upsert_set", {
                id: set_selected?.id ?? null,
                workoutExerciseId: workoutExerciseId,
                weightKg: weightInput,
                reps: repsInput,
                notes: null,
            });
            set_selected = null;
            lastSet = { weight: weightInput, reps: repsInput };
            await refreshSets();
            defaultToLastSet();
        } finally {
            adding = false;
        }
    }

    async function deleteSet() {
        if (!set_selected) return;
        await invoke("delete_set", { id: set_selected.id });
        set_selected = null;
        await refreshSets();
        defaultToLastSet();
    }

    function handleSetConsider(e: CustomEvent) {
        sets = e.detail.items;
    }

    function handleSetFinalize(e: CustomEvent) {
        sets = e.detail.items;
        invoke("reorder_sets", {
            workoutExerciseId,
            orderedSetIds: sets.map((s) => s.id),
        }).then(() => refreshSets());
    }
</script>

<div class="body">
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
                    <span class="drag-handle"
                        ><GripVertical size={16} strokeWidth={1.5} /></span
                    >
                    <span class="set-stats">
                        <span class="stat-val stat-val--weight"
                            >{formatWeight(set.weight_kg)}</span
                        ><span class="stat-unit">kg</span>
                        <span class="stat-val stat-val--reps">{set.reps}</span
                        ><span class="stat-unit">reps</span>
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
                <div class="body-value">
                    <button
                        class="body-btn"
                        onclick={() =>
                            (weightInput = Math.max(weightInput - 2.5, 0))}
                        ><SquareMinus size={20} strokeWidth={1.5} /></button
                    >
                    <input
                        type="number"
                        bind:value={weightInput}
                        onfocus={(e) =>
                            setTimeout(
                                () => (e.target as HTMLInputElement).select(),
                                0,
                            )}
                    />
                    <button
                        class="body-btn"
                        onclick={() => (weightInput += 2.5)}
                        ><SquarePlus size={20} strokeWidth={1.5} /></button
                    >
                </div>
            </div>
            <div class="field">
                <label for="reps">Reps</label>
                <div class="body-value">
                    <button
                        class="body-btn"
                        onclick={() => (repsInput = Math.max(repsInput - 1, 0))}
                        ><SquareMinus size={20} strokeWidth={1.5} /></button
                    >
                    <input
                        type="number"
                        bind:value={repsInput}
                        onfocus={(e) =>
                            setTimeout(
                                () => (e.target as HTMLInputElement).select(),
                                0,
                            )}
                    />
                    <button class="body-btn" onclick={() => (repsInput += 1)}
                        ><SquarePlus size={20} strokeWidth={1.5} /></button
                    >
                </div>
            </div>
        </div>
        {#if set_selected}
            <button class="update-btn" onclick={addSet} disabled={adding}>
                {adding ? "Updating…" : "Update set"}
            </button>
            <button class="delete-btn" onclick={deleteSet}>Delete</button>
        {:else}
            <button class="add-btn" onclick={addSet} disabled={adding}>
                {adding ? "Adding…" : "Add set"}
            </button>
        {/if}
    </div>
</div>
