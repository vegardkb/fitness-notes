<script lang="ts">
    import { page } from "$app/state";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import ExerciseHeader from "$lib/ExerciseHeader.svelte";
    import type { Exercise } from "$lib/exercise";
    import { exerciseHrefs } from "$lib/exercise";

    let { children } = $props();

    const date = $derived(
        page.params.date ?? page.url.searchParams.get("from") ?? "",
    );
    const exerciseId = $derived(Number(page.params.id));

    const activeTab = $derived(
        page.route.id?.endsWith("/history")
            ? "history"
            : page.route.id?.endsWith("/graph")
              ? "graph"
              : page.route.id?.endsWith("/prs")
                ? "prs"
                : "sets",
    );

    const hrefs = $derived(exerciseHrefs(exerciseId, date));

    let exerciseName: string = $state("");

    onMount(async () => {
        const exercise = await invoke<Exercise>("get_exercise", {
            id: exerciseId,
        });
        exerciseName = exercise.name;
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
        {activeTab}
        {date}
    />
    {@render children()}
</div>
