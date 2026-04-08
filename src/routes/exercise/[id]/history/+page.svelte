<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { formatDate } from "$lib/date";
    import type { DayWorkout } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";

    const exerciseId = $derived(Number(page.params.id ?? "0"));

    let history = $state<DayWorkout[]>([]);

    onMount(async () => {
        history = await invoke<DayWorkout[]>("get_exercise_history", {
            exerciseId,
        });
    });
</script>

<div class="body">
    {#if history.length === 0}
        <p class="empty">No sessions logged yet.</p>
    {:else}
        <div class="list">
            {#each history as day}
                <div class="exercise-card">
                    <button
                        class="exercise-card-header"
                        onclick={() =>
                            goto(`/exercise/${exerciseId}/${day.date}`)}
                    >
                        <span>{formatDate(day.date)}</span>
                        <span class="muted">→</span>
                    </button>
                    <div class="exercise-card-sets">
                        {#each day.exercises[0].sets as set, i}
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
</div>
