<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import type { RepMax } from "$lib/exercise";
    import { formatDate } from "$lib/date";
    import ExerciseHeader from "$lib/ExerciseHeader.svelte";
    import { exerciseHrefs, formatWeight } from "$lib/exercise";

    type Exercise = { id: number; name: string };

    const exerciseId = $derived(Number(page.params.id ?? "0"));
    const date = $derived(page.url.searchParams.get("from") ?? "");
    const hrefs = $derived(exerciseHrefs(exerciseId, date));

    let exerciseName = $state("");
    let repMaxes = $state<RepMax[]>([]);

    onMount(async () => {
        const [repMaxesData, exercise] = await Promise.all([
            invoke<RepMax[]>("get_rep_maxes", { exerciseId }),
            invoke<Exercise>("get_exercise", { id: exerciseId }),
        ]);
        exerciseName = exercise.name;
        repMaxes = repMaxesData;
    });
    const filled = $derived(() => {
        const byReps = new Map(repMaxes.map((r) => [r.reps, r]));
        const n = Math.max(...repMaxes.map((r) => r.reps), 0);
        return Array.from({ length: n }, (_, i) => {
            const k = i + 1;
            if (byReps.has(k)) return { ...byReps.get(k)!, ghost: false };
            // find closest l > k
            for (let l = k + 1; l <= n; l++) {
                if (byReps.has(l))
                    return { ...byReps.get(l)!, ghost: true, reps: k };
            }
            return { ...byReps.get(n)!, ghost: true, reps: k };
        }).filter(Boolean);
    });
    let repMaxesFilled = $derived(filled());
</script>

<div class="page">
    <ExerciseHeader
        feedHref={hrefs.feedHref}
        setsHref={hrefs.setsHref}
        historyHref={hrefs.historyHref}
        graphHref={hrefs.graphHref}
        prsHref={hrefs.prsHref}
        {exerciseName}
        activeTab="prs"
    />
    {#if repMaxes.length === 0}
        <p class="empty">No PRs for this exercise.</p>
    {:else}
        <table class="pr-table">
            <thead>
                <tr>
                    <th class="th-rm">PRs</th>
                </tr>
            </thead>
            <tbody>
                {#each repMaxesFilled as item}
                    <tr class:ghost={item.ghost}>
                        <td class="td-reps">{item.reps}</td>
                        <td class="td-rm">RM</td>
                        <td class="td-weight">
                            <span>{formatWeight(item.weight_kg)} kg</span>
                            <span class="date"
                                >{new Intl.DateTimeFormat("no", {
                                    dateStyle: "short",
                                }).format(
                                    new Date(item.date + "T00:00:00"),
                                )}</span
                            >
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>
