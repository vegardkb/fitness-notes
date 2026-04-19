<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "$lib/tauri";
    import { onMount } from "svelte";
    import { bodyHrefs } from "$lib/body";

    import BodyHeader from "$lib/BodyHeader.svelte";
    import type { Measurement, Metric } from "$lib/body";
    import { SquareMinus, SquarePlus, Circle } from "lucide-svelte";

    const date = $derived(page.params.date ?? "");
    const hrefs = $derived(bodyHrefs(date));

    let measurements = $state<Measurement[]>([]);
    let measurementsFilled = $state<Measurement[]>([]);
    let metrics = $state<Metric[]>([]);

    onMount(async () => {
        await fetchMeasurements();
    });

    async function fetchMeasurements() {
        [measurements, metrics] = await Promise.all([
            invoke<Measurement[]>("get_last_measurements_for_date", { date }),
            invoke<Metric[]>("list_metrics"),
        ]);
        measurementsFilled = metrics.map((metric) => ({
            value: round(
                measurements.find((m) => m.metric.id === metric.id)?.value ??
                    NaN,
            ),
            metric,
            date:
                measurements.find((m) => m.metric.id === metric.id)?.date ??
                date,
            id:
                measurements.find(
                    (m) => m.metric.id === metric.id && m.date === date,
                )?.id ?? null,
        }));
        // sort by is_derived and then name
        measurementsFilled = measurementsFilled.sort(
            (a, b) =>
                Number(b.metric.is_derived) - Number(a.metric.is_derived) ||
                a.metric.name.localeCompare(b.metric.name),
        );
    }

    function round(value: number): number {
        return Math.round(value * 100) / 100;
    }

    async function toggle(row: Measurement) {
        if (row.metric.is_derived) {
            return;
        }
        if (row.id != null) {
            await deleteMeasurement(row);
        } else {
            await save(row);
        }
    }

    async function incDec(row: Measurement, delta: number) {
        if (row.metric.is_derived) {
            return;
        }
        const base = isNaN(row.value) ? 0 : row.value;
        row.value = round(Math.max(base + delta, 0));
        save(row);
    }

    async function save(row: Measurement) {
        if (row.metric.is_derived) {
            return;
        }
        let i = measurementsFilled.findIndex((m) => m.id === row.id);
        let id = await invoke<number>("upsert_body_measurement", {
            id: row.id,
            date: date,
            measureId: row.metric.id,
            value: row.value,
        });
        measurementsFilled[i] = { ...measurementsFilled[i], id };
        await fetchMeasurements();
    }

    async function deleteMeasurement(row: Measurement) {
        if (row.metric.is_derived) {
            return;
        }
        let i = measurementsFilled.findIndex((m) => m.id === row.id);
        if (row.id != null) {
            await invoke<void>("delete_body_measurement", { id: row.id });
            measurementsFilled[i] = { ...measurementsFilled[i], id: null };
            await fetchMeasurements();
        }
    }

    function muteRow(row: Measurement) {
        return (
            (row.id === null && !row.metric.is_derived) ||
            (row.metric.is_derived && (isNaN(row.value) || row.date != date))
        );
    }
</script>

<div class="page">
    <BodyHeader
        feedHref={hrefs.feedHref}
        logHref={hrefs.logHref}
        historyHref={hrefs.historyHref}
        graphHref={hrefs.graphHref}
        prsHref={hrefs.prsHref}
        activeTab="log"
        {date}
    />

    <table class="body-grid">
        <thead>
            <tr class="body-row">
                <th scope="col">Metric</th>
                <th scope="col">Value</th>
                <th scope="col">Save</th>
            </tr>
        </thead>
        <tbody>
            {#each measurementsFilled as row}
                <tr class="body-row" class:body-row--unsaved={muteRow(row)}>
                    {#if row.metric.is_derived}
                        <td
                            ><span class="set-label">{row.metric.name}</span>
                            <span class="stat-unit">{row.metric.unit}</span></td
                        >
                        <td class="body-value">
                            <span class="stat-val">
                                {isNaN(row.value) ? "—" : row.value}
                            </span>
                        </td>
                        <td></td>
                    {:else}
                        <td
                            ><span>{row.metric.name}</span>
                            <span class="body-unit">{row.metric.unit}</span></td
                        >
                        <td class="body-value">
                            <button
                                class="body-btn"
                                onclick={() => incDec(row, -0.1)}
                                ><SquareMinus
                                    size={20}
                                    strokeWidth={1.5}
                                /></button
                            >
                            <div class="field">
                                <span class="stat-val">
                                    <input
                                        type="number"
                                        bind:value={row.value}
                                        onblur={() => save(row)}
                                    /></span
                                >
                            </div>
                            <button
                                class="body-btn"
                                onclick={() => incDec(row, 0.1)}
                                ><SquarePlus
                                    size={20}
                                    strokeWidth={1.5}
                                /></button
                            >
                        </td>
                        <td>
                            <button
                                class="save-btn"
                                title="Save"
                                class:save-btn--saved={row.id !== null}
                                onclick={() => toggle(row)}
                            >
                                <Circle size={20} strokeWidth={1.5} />
                            </button>
                        </td>
                    {/if}
                </tr>
            {/each}
        </tbody>
    </table>
</div>
