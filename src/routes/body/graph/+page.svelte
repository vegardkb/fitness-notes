<script lang="ts">
    import { LayerCake, Svg } from "layercake";
    import { scaleTime, scaleLinear } from "d3-scale";
    import { page } from "$app/state";
    import { invoke } from "$lib/tauri";
    import { onMount } from "svelte";
    import { todayStr, formatDateLong } from "$lib/date";
    import { formatWeight } from "$lib/exercise";
    import type { Metric } from "$lib/body";
    import { bodyHrefs } from "$lib/body";
    import BodyHeader from "$lib/BodyHeader.svelte";

    import Line from "$lib/chart/Line.svelte";
    import Points from "$lib/chart/Points.svelte";
    import AxisX from "$lib/chart/AxisX.svelte";
    import AxisY from "$lib/chart/AxisY.svelte";

    type DataPoint = { date: string; value: number };

    function offsetMonths(dateStr: string, months: number): string {
        const [y, m, d] = dateStr.split("-").map(Number);
        const dt = new Date(y, m - 1 + months, d);
        return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, "0")}-${String(dt.getDate()).padStart(2, "0")}`;
    }

    const date = $derived(page.url.searchParams.get("from") ?? "");
    const hrefs = $derived(bodyHrefs(date));

    let exerciseName = "Body Tracker";
    let data = $state<DataPoint[]>([]);
    let loading = $state(false);

    // Date range
    type Range = "1M" | "1Y" | "3Y" | "All";
    let range = $state<Range>("1M");

    const fromDate = $derived.by(() => {
        const today = todayStr();
        if (range === "1M") return offsetMonths(today, -1);
        if (range === "1Y") return offsetMonths(today, -12);
        if (range === "3Y") return offsetMonths(today, -36);
        return "2000-01-01";
    });

    const toDate = $derived(todayStr());

    // Metric switcher
    let metrics = $state<Metric[]>([]);
    let selectedMetric = $state<number | undefined>(undefined);

    // Load graph data whenever exerciseId or date range changes
    $effect(() => {
        const id = selectedMetric;

        const from = fromDate;
        const to = toDate;
        loading = true;
        invoke<DataPoint[]>("get_measurements_graph_data", {
            metricId: id,
            fromDate: from,
            toDate: to,
        })
            .then((d) => {
                data = d;
            })
            .finally(() => {
                loading = false;
            });
    });

    onMount(async () => {
        metrics = await invoke<Metric[]>("list_metrics");
    });

    // Tooltip state
    let tooltip = $state<{
        x: number;
        y: number;
        date: string;
        metric: number;
    } | null>(null);
</script>

<div class="body">
    <BodyHeader
        feedHref={hrefs.feedHref}
        logHref={hrefs.logHref}
        historyHref={hrefs.historyHref}
        graphHref={hrefs.graphHref}
        prsHref={hrefs.prsHref}
        activeTab="history"
        {date}
    />

    <div class="graph-ranges">
        {#each ["1M", "1Y", "3Y", "All"] as const as r}
            <button
                class="range-pill"
                class:range-pill--active={range === r}
                onclick={() => (range = r)}
            >
                {r}
            </button>
        {/each}
    </div>

    <div class="graph-switcher">
        <select bind:value={selectedMetric} aria-label="BodyMetric">
            <option value="">Metric…</option>
            {#each metrics as metric}
                <option value={metric.id}>{metric.name}</option>
            {/each}
        </select>
    </div>

    {#if loading}
        <p class="empty">Loading…</p>
    {:else if data.length === 0}
        <p class="empty">No data for this period.</p>
    {:else}
        <div class="graph-chart">
            <LayerCake
                {data}
                x={(d: DataPoint) => new Date(d.date)}
                y="value"
                xScale={scaleTime()}
                yScale={scaleLinear()}
                padding={{ top: 12, right: 16, bottom: 28, left: 44 }}
                yNice
            >
                <Svg label="1RM chart for {exerciseName}">
                    <AxisY format={(v) => formatWeight(v)} />
                    <AxisX />
                    <Line />
                    <Points
                        onpointclick={(d: DataPoint, e: MouseEvent) => {
                            tooltip = {
                                x: e.clientX + 10,
                                y: e.clientY - 30,
                                date: d.date,
                                metric: d.value,
                            };
                        }}
                        onpointenter={(d: DataPoint, e: PointerEvent) => {
                            tooltip = {
                                x: e.clientX + 10,
                                y: e.clientY - 30,
                                date: d.date,
                                metric: d.value,
                            };
                        }}
                        onpointleave={() => (tooltip = null)}
                    />
                </Svg>
            </LayerCake>
        </div>
    {/if}
</div>

{#if tooltip}
    <div class="graph-tooltip" style="left: {tooltip.x}px; top: {tooltip.y}px;">
        <span style="color: var(--text-muted); margin-right: 0.4rem;"
            >{formatDateLong(tooltip.date)}</span
        >
        <strong>{formatWeight(tooltip.metric)} kg</strong>
    </div>
{/if}
