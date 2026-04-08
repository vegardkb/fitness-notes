<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { dndzone } from "svelte-dnd-action";
    import type { Set, ExerciseWithSets, SetMinimal } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";

    const exerciseId = $derived(Number(page.params.id ?? "0"));
    const date = $derived(page.params.date ?? "");

    let sets = $state<Set[]>([]);
    let weightInput: number = $state(NaN);
    let repsInput: number = $state(NaN);
    let adding = $state(false);
    let set_selected = $state<Set | null>(null);
    let lastSet: SetMinimal | null = $state(null);

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
        const workout = await invoke<ExerciseWithSets[]>(
            "get_workout_for_date",
            { date },
        );
        sets = workout.find((e) => e.exercise_id === exerciseId)?.sets ?? [];
    }

    onMount(async () => {
        const [workout, lastSetData] = await Promise.all([
            invoke<ExerciseWithSets[]>("get_workout_for_date", { date }),
            invoke<SetMinimal>("get_last_set", { exerciseId }),
        ]);

        lastSet = lastSetData;
        sets = workout.find((e) => e.exercise_id === exerciseId)?.sets ?? [];
        defaultToLastSet();
    });

    async function addSet() {
        if (!weightInput || !repsInput) return;
        adding = true;
        try {
            await invoke<Set>("upsert_set", {
                id: set_selected?.id ?? null,
                date,
                exerciseId,
                weightKg: weightInput,
                reps: repsInput,
                notes: null,
            });
            set_selected = null;
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
            exerciseId,
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
                    <span class="drag-handle">≡</span>
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
                        >-</button
                    >
                    <input type="number" bind:value={weightInput} />
                    <button
                        class="body-btn"
                        onclick={() => (weightInput += 2.5)}>+</button
                    >
                </div>
            </div>
            <div class="field">
                <label for="reps">Reps</label>
                <div class="body-value">
                    <button
                        class="body-btn"
                        onclick={() => (repsInput = Math.max(repsInput - 1, 0))}
                        >-</button
                    >
                    <input type="number" bind:value={repsInput} />
                    <button class="body-btn" onclick={() => (repsInput += 1)}
                        >+</button
                    >
                </div>
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
