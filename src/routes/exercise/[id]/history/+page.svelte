<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

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

    type DayWorkout = {
        date: string;
        exercises: ExerciseWithSets[];
    };

    type Exercise = {
        id: number;
        name: string;
    };

    const exerciseId = $derived(Number(page.params.id ?? "0"));
    const fromDate = $derived(page.url.searchParams.get("from") ?? "");

    function todayStr(): string {
        const n = new Date();
        return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, "0")}-${String(n.getDate()).padStart(2, "0")}`;
    }

    const feedHref = $derived(fromDate ? `/?date=${fromDate}` : "/");
    const setsHref = $derived(
        fromDate ? `/exercise/${exerciseId}/${fromDate}` : `/exercise/${exerciseId}/${todayStr()}`,
    );
    const graphHref = $derived(
        fromDate
            ? `/exercise/${exerciseId}/graph?from=${fromDate}`
            : `/exercise/${exerciseId}/graph`,
    );

    let exerciseName = $state("");
    let history = $state<DayWorkout[]>([]);

    function formatWeight(kg: number): string {
        const f2 = kg.toFixed(2);
        return f2.endsWith("0") ? kg.toFixed(1) : f2;
    }

    function formatDate(dateStr: string): string {
        const [y, m, d] = dateStr.split("-").map(Number);
        const dt = new Date(y, m - 1, d);
        return dt.toLocaleDateString(undefined, {
            weekday: "long",
            year: "numeric",
            month: "long",
            day: "numeric",
        });
    }

    onMount(async () => {
        const [historyData, exercise] = await Promise.all([
            invoke<DayWorkout[]>("get_exercise_history", { exerciseId }),
            invoke<Exercise>("get_exercise", { id: exerciseId }),
        ]);
        exerciseName = exercise.name;
        history = historyData;
    });
</script>

<div class="page">
    <div class="history-header">
        <a class="back-btn" href={feedHref}>←</a>
        <h1>{exerciseName}</h1>
        <div class="header-tabs">
            <a class="header-tab" href={setsHref} aria-label="Sets">
                <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                    <line x1="4" y1="6" x2="16" y2="6"/>
                    <line x1="4" y1="10" x2="16" y2="10"/>
                    <line x1="4" y1="14" x2="16" y2="14"/>
                </svg>
            </a>
            <span class="header-tab header-tab--active" aria-label="History">
                <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="10" cy="10" r="8"/>
                    <polyline points="10,6 10,10 13,12"/>
                </svg>
            </span>
            <a class="header-tab" href={graphHref} aria-label="Graph">
                <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="2,15 7,9 11,12 18,4"/>
                    <line x1="2" y1="18" x2="18" y2="18"/>
                </svg>
            </a>
        </div>
    </div>

    {#if history.length === 0}
        <p class="empty">No sessions logged yet.</p>
    {:else}
        <div class="list">
            {#each history as day}
                <div class="exercise-card">
                    <button
                        class="exercise-card-header"
                        onclick={() => goto(`/exercise/${exerciseId}/${day.date}`)}
                    >
                        <span>{formatDate(day.date)}</span>
                        <span class="muted">→</span>
                    </button>
                    <div class="exercise-card-sets">
                        {#each day.exercises[0].sets as set, i}
                            <div class="set-row">
                                <span class="set-label">{i + 1}</span>
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
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
