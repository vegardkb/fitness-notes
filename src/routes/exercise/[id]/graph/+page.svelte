<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { todayStr, formatDateLong } from "$lib/date";

    type DataPoint = { date: string; metric: number };
    type Exercise = { id: number; name: string };

    function offsetMonths(dateStr: string, months: number): string {
        const [y, m, d] = dateStr.split("-").map(Number);
        const dt = new Date(y, m - 1 + months, d);
        return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, "0")}-${String(dt.getDate()).padStart(2, "0")}`;
    }

    const exerciseId = $derived(Number(page.params.id ?? "0"));
    const dateCtx = $derived(page.url.searchParams.get("from") ?? "");

    const feedHref = $derived(dateCtx ? `/?date=${dateCtx}` : "/");
    const setsHref = $derived(
        dateCtx
            ? `/exercise/${exerciseId}/${dateCtx}`
            : `/exercise/${exerciseId}/${todayStr()}`,
    );
    const historyHref = $derived(
        dateCtx
            ? `/exercise/${exerciseId}/history?from=${dateCtx}`
            : `/exercise/${exerciseId}/history`,
    );

    let exerciseName = $state("");
    let data = $state<DataPoint[]>([]);
    let loading = $state(false);

    // Date range
    type Range = "1M" | "1Y" | "3Y" | "All";
    let range = $state<Range>("1Y");

    const fromDate = $derived.by(() => {
        const today = todayStr();
        if (range === "1M") return offsetMonths(today, -1);
        if (range === "1Y") return offsetMonths(today, -12);
        if (range === "3Y") return offsetMonths(today, -36);
        return "2000-01-01";
    });

    const toDate = $derived(todayStr());

    // Exercise switcher
    let categories = $state<string[]>([]);
    let selectedCategory = $state("");
    let categoryExercises = $state<Exercise[]>([]);
    let selectedExerciseId = $state("");

    $effect(() => {
        if (!selectedCategory) return;
        invoke<Exercise[]>("list_exercises_in_category", {
            category: selectedCategory,
        }).then((exs) => {
            categoryExercises = exs;
            selectedExerciseId = "";
        });
    });

    $effect(() => {
        const newId = Number(selectedExerciseId);
        if (newId && newId !== exerciseId) {
            selectedExerciseId = "";
            const ctx = dateCtx;
            goto(
                ctx
                    ? `/exercise/${newId}/graph?from=${ctx}`
                    : `/exercise/${newId}/graph`,
            );
        }
    });

    // Update exercise name whenever the exercise changes (e.g. via switcher navigation)
    $effect(() => {
        const id = exerciseId;
        invoke<Exercise>("get_exercise", { id }).then((ex) => {
            exerciseName = ex.name;
        });
    });

    // Load graph data whenever exerciseId or date range changes
    $effect(() => {
        const id = exerciseId;
        const from = fromDate;
        const to = toDate;
        loading = true;
        invoke<DataPoint[]>("get_exercise_graph_data", {
            exerciseId: id,
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
        categories = await invoke<string[]>("list_exercise_categories");
    });

    // ── Chart geometry ────────────────────────────────────────────────────────

    const PAD_LEFT = 50;
    const PAD_RIGHT = 16;
    const PAD_TOP = 16;
    const PAD_BOTTOM = 36;
    const W = 500;
    const H = 220;
    const PLOT_W = W - PAD_LEFT - PAD_RIGHT; // 434
    const PLOT_H = H - PAD_TOP - PAD_BOTTOM; // 168

    function dateToMs(dateStr: string): number {
        const [y, m, d] = dateStr.split("-").map(Number);
        return new Date(y, m - 1, d).getTime();
    }

    const chartPoints = $derived.by(() => {
        if (data.length === 0) return [];
        const minMs = dateToMs(data[0].date);
        const maxMs = dateToMs(data.at(-1)!.date);
        const msRange = maxMs - minMs || 1;
        const minMetric = Math.min(...data.map((d) => d.metric));
        const maxMetric = Math.max(...data.map((d) => d.metric));
        const metricRange = maxMetric - minMetric || 1;

        return data.map((d) => ({
            x: PAD_LEFT + ((dateToMs(d.date) - minMs) / msRange) * PLOT_W,
            y:
                PAD_TOP +
                PLOT_H -
                ((d.metric - minMetric) / metricRange) * PLOT_H,
            date: d.date,
            metric: d.metric,
        }));
    });

    const polylinePoints = $derived(
        chartPoints.map((p) => `${p.x},${p.y}`).join(" "),
    );

    // Grid lines (4 horizontal, evenly spaced in y)
    const gridLines = $derived.by(() => {
        if (data.length === 0) return [];
        const minMetric = Math.min(...data.map((d) => d.metric));
        const maxMetric = Math.max(...data.map((d) => d.metric));
        const metricRange = maxMetric - minMetric || 1;
        return [0, 1, 2, 3].map((i) => {
            const frac = i / 3;
            const y = PAD_TOP + PLOT_H - frac * PLOT_H;
            const kg = minMetric + frac * metricRange;
            return { y, label: kg.toFixed(1) };
        });
    });

    // X-axis labels: up to 5 evenly spaced
    const xLabels = $derived.by(() => {
        if (data.length === 0) return [];
        const n = Math.min(data.length, 5);
        const indices = Array.from({ length: n }, (_, i) =>
            Math.round((i / (n - 1 || 1)) * (data.length - 1)),
        );
        const minMs = dateToMs(data[0].date);
        const maxMs = dateToMs(data.at(-1)!.date);
        const msRange = maxMs - minMs || 1;

        return [...new Set(indices)].map((idx) => {
            const d = data[idx];
            const x =
                PAD_LEFT + ((dateToMs(d.date) - minMs) / msRange) * PLOT_W;
            const [y, m, day] = d.date.split("-").map(Number);
            const label = new Date(y, m - 1, day).toLocaleDateString(
                undefined,
                {
                    month: "short",
                    day: "numeric",
                },
            );
            return { x, label };
        });
    });

    // Tooltip state
    let tooltip = $state<{
        x: number;
        y: number;
        date: string;
        metric: number;
    } | null>(null);

    function formatWeight(kg: number): string {
        const f2 = kg.toFixed(2);
        return f2.endsWith("0") ? kg.toFixed(1) : f2;
    }
</script>

<div class="page">
    <div class="history-header">
        <a class="back-btn" href={feedHref}>←</a>
        <h1>{exerciseName || "…"}</h1>
        <div class="header-tabs">
            <a class="header-tab" href={setsHref} aria-label="Sets">
                <svg
                    width="18"
                    height="18"
                    viewBox="0 0 20 20"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                >
                    <line x1="4" y1="6" x2="16" y2="6" />
                    <line x1="4" y1="10" x2="16" y2="10" />
                    <line x1="4" y1="14" x2="16" y2="14" />
                </svg>
            </a>
            <a class="header-tab" href={historyHref} aria-label="History">
                <svg
                    width="18"
                    height="18"
                    viewBox="0 0 20 20"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <circle cx="10" cy="10" r="8" />
                    <polyline points="10,6 10,10 13,12" />
                </svg>
            </a>
            <span class="header-tab header-tab--active" aria-label="Graph">
                <svg
                    width="18"
                    height="18"
                    viewBox="0 0 20 20"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <polyline points="2,15 7,9 11,12 18,4" />
                    <line x1="2" y1="18" x2="18" y2="18" />
                </svg>
            </span>
        </div>
    </div>

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
        <select bind:value={selectedCategory} aria-label="Category">
            <option value="">Category…</option>
            {#each categories as cat}
                <option value={cat}>{cat}</option>
            {/each}
        </select>
        <select
            bind:value={selectedExerciseId}
            disabled={categoryExercises.length === 0}
            aria-label="Exercise"
        >
            <option value="">Exercise…</option>
            {#each categoryExercises as ex}
                <option value={String(ex.id)}>{ex.name}</option>
            {/each}
        </select>
    </div>

    {#if loading}
        <p class="empty">Loading…</p>
    {:else if data.length === 0}
        <p class="empty">No data for this period.</p>
    {:else}
        <svg
            class="graph-chart"
            viewBox="0 0 {W} {H}"
            role="img"
            aria-label="1RM chart for {exerciseName}"
        >
            <!-- Grid lines + Y labels -->
            {#each gridLines as gl}
                <line
                    x1={PAD_LEFT}
                    y1={gl.y}
                    x2={W - PAD_RIGHT}
                    y2={gl.y}
                    stroke="var(--border)"
                    stroke-dasharray="3 3"
                />
                <text
                    x={PAD_LEFT - 6}
                    y={gl.y + 4}
                    text-anchor="end"
                    fill="var(--text-muted)"
                    font-size="11"
                    font-family="system-ui, sans-serif">{gl.label}</text
                >
            {/each}

            <!-- X labels -->
            {#each xLabels as xl}
                <text
                    x={xl.x}
                    y={H - PAD_BOTTOM + 16}
                    text-anchor="middle"
                    fill="var(--text-muted)"
                    font-size="11"
                    font-family="system-ui, sans-serif">{xl.label}</text
                >
            {/each}

            <!-- Line -->
            {#if chartPoints.length > 1}
                <polyline
                    points={polylinePoints}
                    stroke="var(--accent)"
                    stroke-width="2"
                    fill="none"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                />
            {/if}

            <!-- Dots -->
            {#each chartPoints as pt}
                <circle
                    cx={pt.x}
                    cy={pt.y}
                    r="4"
                    fill="var(--accent)"
                    role="presentation"
                    style="cursor: pointer;"
                    onmouseenter={(e) => {
                        const rect = (e.currentTarget as SVGCircleElement)
                            .closest("svg")!
                            .getBoundingClientRect();
                        const svgX = pt.x / W;
                        const svgY = pt.y / H;
                        tooltip = {
                            x: rect.left + svgX * rect.width + 10,
                            y: rect.top + svgY * rect.height - 30,
                            date: pt.date,
                            metric: pt.metric,
                        };
                    }}
                    onmouseleave={() => (tooltip = null)}
                />
            {/each}
        </svg>
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
