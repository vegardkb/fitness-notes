<script lang="ts">
    import { formatDate } from "$lib/date";
    import {
        ArrowLeft,
        AlignJustify,
        Clock,
        TrendingUp,
        Trophy,
    } from "lucide-svelte";

    let props = $props();
    let exerciseName = $derived(
        props.exerciseName ? props.exerciseName : "Exercise",
    );
    let date = $derived(props.date ? formatDate(props.date) : "");
</script>

<div class="history-header">
    <a class="back-btn" href={props.feedHref}
        ><ArrowLeft size={18} strokeWidth={1.5} /></a
    >
    {#if props.activeTab === "sets"}
        <h1>{exerciseName}</h1>
        <p>{date}</p>
    {:else if props.activeTab === "history"}
        <h1>{exerciseName}</h1>
        <p>History</p>
    {:else if props.activeTab === "graph"}
        <h1>{exerciseName}</h1>
        <p>Graph</p>
    {:else if props.activeTab === "prs"}
        <h1>{exerciseName}</h1>
        <p>PRs</p>
    {:else}
        <h1>{exerciseName}</h1>
    {/if}
    <div class="header-tabs">
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "sets"}
            href={props.setsHref}
            aria-label="Sets"
        >
            <AlignJustify size={18} strokeWidth={1.5} />
        </a>
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "history"}
            href={props.historyHref}
            aria-label="History"
        >
            <Clock size={18} strokeWidth={1.5} />
        </a>
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "graph"}
            href={props.graphHref}
            aria-label="Graph"
        >
            <TrendingUp size={18} strokeWidth={1.5} />
        </a>
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "prs"}
            href={props.prsHref}
            aria-label="PRs"
        >
            <Trophy size={18} strokeWidth={1.5} />
        </a>
    </div>
</div>
