<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { dndzone } from "svelte-dnd-action";
    import { formatWeight, exerciseHrefs } from "$lib/exercise";
    import BodyHeader from "$lib/BodyHeader.svelte";
    import type { Measurement, Metric } from "$lib/body";

    const exerciseId = $derived(Number(page.params.id ?? "0"));
    const date = $derived(page.params.date ?? "");
    const hrefs = $derived(exerciseHrefs(exerciseId, date));

    let exerciseName = $state("");
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
                measurements.find((m) => m.metric.id === metric.id)?.value ?? 0,
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
    }

    function round(value: number): number {
        return Math.round(value * 100) / 100;
    }

    async function toggle(row: Measurement) {
        if (row.id != null) {
            await deleteMeasurement(row);
        } else {
            await save(row);
        }
    }

    async function incDec(row: Measurement, delta: number) {
        row.value = round(row.value + delta);
        save(row);
    }

    async function save(row: Measurement) {
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
        let i = measurementsFilled.findIndex((m) => m.id === row.id);
        if (row.id != null) {
            await invoke<void>("delete_body_measurement", { id: row.id });
            measurementsFilled[i] = { ...measurementsFilled[i], id: null };
            await fetchMeasurements();
        }
    }
</script>

<div class="page">
    <BodyHeader
        feedHref={hrefs.feedHref}
        setsHref={hrefs.setsHref}
        historyHref={hrefs.historyHref}
        graphHref={hrefs.graphHref}
        prsHref={hrefs.prsHref}
        {exerciseName}
        activeTab="sets"
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
                <tr class="body-row" class:body-row--unsaved={row.id === null}>
                    <td>{row.metric.name} ({row.metric.unit})</td>
                    <td class="body-value">
                        <button
                            class="body-btn"
                            onclick={() => incDec(row, -0.1)}>-</button
                        >
                        <input
                            type="number"
                            bind:value={row.value}
                            onblur={() => save(row)}
                        />
                        <button
                            class="body-btn"
                            onclick={() => incDec(row, 0.1)}>+</button
                        ></td
                    >
                    <td>
                        <button
                            class="save-btn"
                            title="Save"
                            class:save-btn--saved={row.id !== null}
                            onclick={() => toggle(row)}
                        >
                            <svg width="20" height="20" viewBox="0 0 20 20">
                                <circle cx="10" cy="10" r="8" />
                            </svg>
                        </button>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
</div>
