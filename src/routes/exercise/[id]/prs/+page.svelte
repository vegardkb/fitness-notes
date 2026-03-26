<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { todayStr, formatDateLong } from "$lib/date";
    import type { RepMax } from "$lib/exercise";
    import ExerciseHeader from "$lib/ExerciseHeader.svelte";
    import { exerciseHrefs } from "$lib/exercise";

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
</div>
