<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "$lib/tauri";
    import { onMount } from "svelte";
    import type { Measurement, Metric } from "$lib/body";
    import { formatWeight } from "$lib/exercise";

    const exerciseId = $derived(Number(page.params.id ?? "0"));

    type MetricRange = {
        metric: Metric;
        min: Measurement | null;
        max: Measurement | null;
    };

    let bodyRangesData = $state<MetricRange[]>([]);
    let bodyRanges = $derived(
        bodyRangesData.filter(
            (range) => range.min !== null && range.max !== null,
        ),
    );

    onMount(async () => {
        bodyRangesData = await invoke<MetricRange[]>("get_body_min_maxes", {});
        console.log(bodyRangesData);
    });
</script>

<div class="body">
    {#if bodyRanges.length === 0}
        <p class="empty">No PRs for this exercise.</p>
    {:else}
        <table class="pr-table">
            <thead>
                <tr>
                    <th class="th-rm">PRs</th>
                </tr>
            </thead>
            <tbody>
                {#each bodyRanges as item}
                    <tr>
                        <td class="td-reps"
                            >{item.metric.name}<span class="stat-unit"
                                >{item.metric.unit}</span
                            ></td
                        >
                        <td class="td-weight">
                            <a>
                                <span class="stat-val stat-val--weight"
                                    >{formatWeight(item.min?.value ?? 0)}
                                </span></a
                            >
                            <span class="date"
                                >{new Intl.DateTimeFormat("no", {
                                    dateStyle: "short",
                                }).format(
                                    new Date(item.min?.date + "T00:00:00"),
                                )}</span
                            >
                        </td>
                        <td class="td-weight">
                            <a>
                                <span class="stat-val stat-val--weight"
                                    >{formatWeight(item.max?.value ?? 0)}
                                </span></a
                            >
                            <span class="date"
                                >{new Intl.DateTimeFormat("no", {
                                    dateStyle: "short",
                                }).format(
                                    new Date(item.max?.date + "T00:00:00"),
                                )}</span
                            >
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>
