<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "$lib/tauri";
    import { onMount } from "svelte";
    import { formatDate } from "$lib/date";
    import type { DayMeasurement } from "$lib/body";
    import { ChevronRight } from "lucide-svelte";

    let history = $state<DayMeasurement[]>([]);

    onMount(async () => {
        history = await invoke<DayMeasurement[]>("get_measurement_history", {});
    });
</script>

<div class="page">
    {#if history.length === 0}
        <p class="empty">No sessions logged yet.</p>
    {:else}
        <div class="list">
            {#each history as day}
                <div class="exercise-card">
                    <button
                        class="exercise-card-header"
                        onclick={() => goto(`/body/${day.date}`)}
                    >
                        <span>{formatDate(day.date)}</span>
                        <span class="muted"
                            ><ChevronRight size={16} strokeWidth={1.5} /></span
                        >
                    </button>
                    <div class="exercise-card-sets">
                        {#each day.measurements as measurement}
                            <div class="set-row">
                                <span class="set-label" style="min-width: 6rem"
                                    >{measurement.metric.name}</span
                                >
                                <span class="set-stats">
                                    <span
                                        class="stat-val"
                                        style="min-width: 6rem"
                                        >{parseFloat(
                                            measurement.value.toFixed(2),
                                        )}</span
                                    >
                                    <span class="stat-unit"
                                        >{measurement.metric.unit}</span
                                    >
                                </span>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
