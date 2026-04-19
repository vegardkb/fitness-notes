<script lang="ts">
    import type { NamedId } from "$lib/exercise";
    import type { Metric } from "$lib/body";
    import { invoke } from "$lib/tauri";
    import { onMount } from "svelte";

    import { ArrowLeft, ChevronRight } from "lucide-svelte";
    import type { Settings } from "$lib/settings";

    type ExerciseRow = {
        date: string;
        exercise_name: string;
        category_name: string;
        weight_kg: number;
        reps: number;
        exercise_id: number | null;
    };
    type BodyRow = {
        date: string;
        measurement: string;
        value: number;
        unit: string;
        metric_id: number | null;
    };

    type UnknownExercise = { csv_name: string; csv_category: string };
    type ParseResult = {
        rows: ExerciseRow[];
        unknown_exercises: UnknownExercise[];
    };
    type ParseBodyResult = {
        rows: BodyRow[];
        unknown_metrics: string[];
    };
    type Resolution =
        | { action: "create" }
        | { action: "map"; id: number; name: string }
        | { action: "skip" };

    type ImportResult = { sets_imported: number; workouts_touched: number };
    type BodyImportResult = {
        measurements_imported: number;
        days_touched: number;
    };

    type Phase =
        | { name: "idle" }
        | {
              name: "resolving";
              rows: ExerciseRow[];
              unknowns: UnknownExercise[];
              resolutions: Record<string, Resolution>;
          }
        | {
              name: "confirming";
              resolvedRows: ExerciseRow[];
              rowCount: number;
              dateCount: number;
          }
        | { name: "importing" }
        | { name: "done"; result: ImportResult }
        | { name: "error"; message: string }
        | {
              name: "body_resolving";
              rows: BodyRow[];
              unknowns: string[];
              resolutions: Record<string, Resolution>;
          }
        | {
              name: "body_confirming";
              resolvedRows: BodyRow[];
              rowCount: number;
              dateCount: number;
          }
        | { name: "body_importing" }
        | { name: "body_done"; result: BodyImportResult };

    let phase = $state<Phase>({ name: "idle" });
    let fileInput = $state<HTMLInputElement>();
    let bodyFileInput = $state<HTMLInputElement>();

    let confirmingDelete = $state(false);

    async function deleteAllData() {
        await invoke("delete_all_data");
        confirmingDelete = false;
    }

    // Inline exercise picker state
    let pickingFor = $state<string | null>(null);
    let pickerView = $state<"categories" | "exercises">("categories");
    let pickerCategories = $state<NamedId[]>([]);
    let pickerExercises = $state<NamedId[]>([]);

    // Inline body metric picker state
    let bodyPickingFor = $state<string | null>(null);
    let bodyMetrics = $state<Metric[]>([]);

    function buildResolvedRows(
        rows: ExerciseRow[],
        resolutions: Record<string, Resolution>,
    ): ExerciseRow[] {
        return rows.map((row) => {
            if (row.exercise_id !== null) {
                return row;
            }
            const res = resolutions[row.exercise_name];
            if (res.action === "map") {
                return {
                    ...row,
                    exercise_id: res.id,
                    exercise_name: res.name,
                    category_name: "",
                };
            }
            return {
                ...row,
                exercise_id: null,
            };
        });
    }

    function buildResolvedBodyRows(
        rows: BodyRow[],
        resolutions: Record<string, Resolution>,
    ): BodyRow[] {
        return rows
            .map((row) => {
                if (row.metric_id !== null) {
                    return row;
                }
                const res = resolutions[row.measurement];
                if (res.action === "map") {
                    return {
                        ...row,
                        measurement: res.name,
                        metric_id: res.id,
                    };
                } else if (res.action === "skip") {
                    return undefined;
                } else {
                    return { ...row, metric_id: null };
                }
            })
            .filter((row) => row !== undefined);
    }

    async function handleFile(e: Event) {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (!file) return;
        const text = await file.text();
        // Reset file input so the same file can be re-selected
        if (fileInput) fileInput.value = "";

        try {
            const result = await invoke<ParseResult>("parse_fitnotes_csv", {
                csvText: text,
            });
            if (result.unknown_exercises.length === 0) {
                const resolvedRows = buildResolvedRows(result.rows, {});
                const dateCount = new Set(resolvedRows.map((r) => r.date)).size;
                phase = {
                    name: "confirming",
                    resolvedRows,
                    rowCount: resolvedRows.length,
                    dateCount,
                };
            } else {
                const resolutions: Record<string, Resolution> = {};
                for (const u of result.unknown_exercises) {
                    resolutions[u.csv_name] = { action: "create" };
                }
                phase = {
                    name: "resolving",
                    rows: result.rows,
                    unknowns: result.unknown_exercises,
                    resolutions,
                };
            }
        } catch (err) {
            phase = { name: "error", message: String(err) };
        }
    }

    async function handleBodyFile(event: Event) {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) return;
        const text = await file.text();
        if (bodyFileInput) bodyFileInput.value = "";
        try {
            const result = await invoke<ParseBodyResult>(
                "parse_body_measurements_csv",
                {
                    csv: text,
                },
            );
            if (result.unknown_metrics.length === 0) {
                const resolvedRows = result.rows;
                const dateCount = new Set(resolvedRows.map((r) => r.date)).size;
                phase = {
                    name: "body_confirming",
                    resolvedRows,
                    rowCount: resolvedRows.length,
                    dateCount,
                };
            } else {
                const resolutions: Record<string, Resolution> = {};
                for (const u of result.unknown_metrics) {
                    resolutions[u] = { action: "create" };
                }
                phase = {
                    name: "body_resolving",
                    rows: result.rows,
                    unknowns: result.unknown_metrics,
                    resolutions,
                };
            }
        } catch (err) {
            phase = { name: "error", message: String(err) };
        }
    }

    function continueToConfirm() {
        if (phase.name !== "resolving") return;
        const resolvedRows = buildResolvedRows(phase.rows, phase.resolutions);
        const dateCount = new Set(resolvedRows.map((r) => r.date)).size;
        phase = {
            name: "confirming",
            resolvedRows,
            rowCount: resolvedRows.length,
            dateCount,
        };
    }

    async function confirmImport() {
        if (phase.name !== "confirming") return;
        const rows = phase.resolvedRows;
        phase = { name: "importing" };
        try {
            const result = await invoke<ImportResult>("import_fitnotes_rows", {
                rows,
            });
            phase = { name: "done", result };
        } catch (err) {
            phase = { name: "error", message: String(err) };
        }
    }

    function reset() {
        phase = { name: "idle" };
        pickingFor = null;
        bodyPickingFor = null;
    }

    async function startBodyPicking(metricName: string) {
        if (bodyPickingFor === metricName) {
            bodyPickingFor = null;
            return;
        }
        bodyPickingFor = metricName;
        bodyMetrics = await invoke<Metric[]>("list_metrics");
    }

    function setBodyResolution(metricName: string, res: Resolution) {
        if (phase.name !== "body_resolving") return;
        phase.resolutions = { ...phase.resolutions, [metricName]: res };
        if (res.action !== "map") bodyPickingFor = null;
    }

    function bodyPickerSelectMetric(m: Metric) {
        if (phase.name !== "body_resolving" || !bodyPickingFor) return;
        phase.resolutions = {
            ...phase.resolutions,
            [bodyPickingFor]: { action: "map", id: m.id, name: m.name },
        };
        bodyPickingFor = null;
    }

    function continueBodyToConfirm() {
        if (phase.name !== "body_resolving") return;
        const resolvedRows = buildResolvedBodyRows(
            phase.rows,
            phase.resolutions,
        );
        const dateCount = new Set(resolvedRows.map((r) => r.date)).size;
        phase = {
            name: "body_confirming",
            resolvedRows,
            rowCount: resolvedRows.length,
            dateCount,
        };
    }

    async function confirmBodyImport() {
        if (phase.name !== "body_confirming") return;
        const rows = phase.resolvedRows;
        phase = { name: "body_importing" };
        try {
            const result = await invoke<BodyImportResult>(
                "import_body_measurement_rows",
                { rows },
            );
            phase = { name: "body_done", result };
        } catch (err) {
            phase = { name: "error", message: String(err) };
        }
    }

    async function startPicking(csvName: string) {
        if (pickingFor === csvName) {
            pickingFor = null;
            return;
        }
        pickingFor = csvName;
        pickerView = "categories";
        pickerCategories = await invoke<NamedId[]>("list_exercise_categories");
        pickerExercises = [];
    }

    async function pickerSelectCategory(cat: NamedId) {
        pickerView = "exercises";
        pickerExercises = await invoke<NamedId[]>(
            "list_exercises_in_category",
            {
                categoryId: cat.id,
            },
        );
    }

    function pickerSelectExercise(ex: NamedId) {
        if (phase.name !== "resolving" || !pickingFor) return;
        phase.resolutions[pickingFor] = {
            action: "map",
            id: ex.id,
            name: ex.name,
        };
        // Svelte 5: trigger reactivity on the record
        phase.resolutions = { ...phase.resolutions };
        pickingFor = null;
    }

    function setResolution(csvName: string, res: Resolution) {
        if (phase.name !== "resolving") return;
        phase.resolutions = { ...phase.resolutions, [csvName]: res };
        if (res.action === "create" && pickingFor === csvName) {
            pickingFor = null;
        }
    }

    let settings: Settings = $state({
        dark_mode: false,
        height: 0,
        sex: "",
        weight: 0,
    });
    let theme = $derived(settings.dark_mode ? "Dark" : "Light");
    async function refreshSettings() {
        settings = await invoke("get_settings");
        document.documentElement.classList.toggle("dark", settings.dark_mode);
    }
    onMount(async () => {
        await refreshSettings();
    });

    async function setTheme(theme: string) {
        await invoke("set_dark_mode", { darkMode: theme === "Dark" });
        await refreshSettings();
    }

    async function setSex(sex: string) {
        await invoke("set_sex", { sex });
        await refreshSettings();
    }

    async function setHeight(height: number) {
        await invoke("set_height", { height });
        await refreshSettings();
    }
</script>

<div class="page">
    <div class="history-header">
        <a class="back-btn" href="/"
            ><ArrowLeft size={18} strokeWidth={1.5} /></a
        >
        <h1>Settings</h1>
    </div>

    {#if phase.name === "idle"}
        <div class="settings-section">
            <h2 class="settings-section-title">Preferences</h2>
            <div class="settings-row">
                <span>Theme</span>
                <div class="graph-ranges">
                    {#each ["Light", "Dark"] as const as s}
                        <button
                            class="range-pill"
                            class:range-pill--active={theme === s}
                            onclick={() => setTheme(s)}
                        >
                            {s}
                        </button>
                    {/each}
                </div>
            </div>
            <div class="settings-row">
                <span>Sex</span>
                <div class="graph-ranges">
                    {#each ["Female", "Male"] as const as s}
                        <button
                            class="range-pill"
                            class:range-pill--active={settings.sex === s}
                            onclick={() => setSex(s)}
                        >
                            {s}
                        </button>
                    {/each}
                </div>
            </div>
            <div class="settings-row">
                <span>Height</span>
                <span class="body-value">
                    <input
                        type="number"
                        step="1"
                        bind:value={settings.height}
                        onchange={() => setHeight(settings.height)}
                        onblur={() => setHeight(settings.height)}
                    />
                    <span class="body-unit">cm</span>
                </span>
            </div>
        </div>
        <div class="settings-section">
            <h2 class="settings-section-title">Import</h2>
            <div class="settings-row">
                <span>FitNotes CSV</span>
                <button
                    class="add-btn-inline"
                    onclick={() => fileInput?.click()}
                >
                    Import CSV
                </button>
            </div>
            <div class="settings-row">
                <span>FitNotes Body Tracker CSV</span>
                <button
                    class="add-btn-inline"
                    onclick={() => bodyFileInput?.click()}
                >
                    Import CSV
                </button>
            </div>
        </div>
        <input
            type="file"
            accept=".csv,text/csv,text/comma-separated-values,application/csv,application/vnd.ms-excel,text/plain"
            bind:this={fileInput}
            onchange={handleFile}
            style="display:none"
        />
        <input
            type="file"
            accept=".csv,text/csv,text/comma-separated-values,application/csv,application/vnd.ms-excel,text/plain"
            bind:this={bodyFileInput}
            onchange={handleBodyFile}
            style="display:none"
        />

        <div class="settings-section">
            <h2 class="settings-section-title">Data</h2>
            {#if confirmingDelete}
                <div
                    class="settings-row"
                    style="flex-direction: column; align-items: stretch; gap: 0.5rem;"
                >
                    <span style="font-size: 0.9rem;"
                        >Delete all workouts and sets? This cannot be undone.</span
                    >
                    <div style="display: flex; gap: 0.5rem;">
                        <button
                            class="delete-btn"
                            style="flex:1;"
                            onclick={deleteAllData}
                        >
                            Delete everything
                        </button>
                        <button
                            class="update-btn"
                            style="flex:1;"
                            onclick={() => (confirmingDelete = false)}
                        >
                            Cancel
                        </button>
                    </div>
                </div>
            {:else}
                <div class="settings-row">
                    <span>Delete all data</span>
                    <button
                        class="add-btn-inline"
                        style="border-color: var(--danger); color: var(--danger);"
                        onclick={() => (confirmingDelete = true)}
                    >
                        Delete
                    </button>
                </div>
            {/if}
        </div>
    {:else if phase.name === "resolving"}
        <p class="settings-section-title">
            {phase.unknowns.length} exercise{phase.unknowns.length === 1
                ? ""
                : "s"} not recognized
        </p>
        <div class="import-unknown-list">
            {#each phase.unknowns as u}
                {@const res = phase.resolutions[u.csv_name]}
                <div
                    class="import-unknown-item"
                    class:is-picking={pickingFor === u.csv_name}
                >
                    <div>
                        <div class="import-unknown-name">{u.csv_name}</div>
                        <div class="import-unknown-meta">
                            Category in CSV: {u.csv_category}
                        </div>
                    </div>
                    <div class="import-unknown-actions">
                        <button
                            class="range-pill"
                            class:range-pill--active={res.action === "create"}
                            onclick={() =>
                                setResolution(u.csv_name, { action: "create" })}
                        >
                            Create new
                        </button>
                        <button
                            class="range-pill"
                            class:range-pill--active={res.action === "map" ||
                                pickingFor === u.csv_name}
                            onclick={() => startPicking(u.csv_name)}
                        >
                            Map to existing
                        </button>
                        {#if res.action === "map"}
                            <span class="import-mapped-label">→ {res.name}</span
                            >
                        {/if}
                    </div>

                    {#if pickingFor === u.csv_name}
                        <div class="import-picker">
                            {#if pickerView === "categories"}
                                <p class="import-unknown-meta">
                                    Select category
                                </p>
                                <div class="list">
                                    {#each pickerCategories as cat}
                                        <button
                                            class="list-item"
                                            onclick={() =>
                                                pickerSelectCategory(cat)}
                                        >
                                            {cat.name}
                                        </button>
                                    {/each}
                                </div>
                            {:else}
                                <div
                                    style="display:flex; gap:0.5rem; align-items:center;"
                                >
                                    <button
                                        class="back-btn"
                                        onclick={() =>
                                            (pickerView = "categories")}
                                        ><ArrowLeft
                                            size={18}
                                            strokeWidth={1.5}
                                        /></button
                                    >
                                    <p class="import-unknown-meta">
                                        Select exercise
                                    </p>
                                </div>
                                <div class="list">
                                    {#each pickerExercises as ex}
                                        <button
                                            class="list-item"
                                            onclick={() =>
                                                pickerSelectExercise(ex)}
                                        >
                                            {ex.name}
                                        </button>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
        <button
            class="add-btn"
            onclick={continueToConfirm}
            style="display:flex;align-items:center;justify-content:center;gap:0.4rem;"
            >Continue <ChevronRight size={16} strokeWidth={1.5} /></button
        >
        <button class="delete-btn" onclick={reset}>Cancel</button>
    {:else if phase.name === "confirming"}
        <div class="import-summary">
            Import <strong>{phase.rowCount} sets</strong> across
            <strong>{phase.dateCount} dates</strong>?
        </div>
        <button class="add-btn" onclick={confirmImport}>Confirm import</button>
        <button class="delete-btn" onclick={reset}>Cancel</button>
    {:else if phase.name === "importing"}
        <p class="empty">Importing…</p>
    {:else if phase.name === "done"}
        <div class="import-summary">
            Imported <strong>{phase.result.sets_imported} sets</strong> across
            <strong>{phase.result.workouts_touched} workouts</strong>.
        </div>
        <button class="add-btn" onclick={reset}>Import another</button>
    {:else if phase.name === "error"}
        <div
            class="import-summary"
            style="border-color: var(--danger); color: var(--danger);"
        >
            {phase.message}
        </div>
        <button class="add-btn" onclick={reset}>Try again</button>
    {:else if phase.name === "body_resolving"}
        <p class="settings-section-title">
            {phase.unknowns.length} metric{phase.unknowns.length === 1
                ? ""
                : "s"} not recognized
        </p>
        <div class="import-unknown-list">
            {#each phase.unknowns as u}
                {@const res = phase.resolutions[u]}
                <div
                    class="import-unknown-item"
                    class:is-picking={bodyPickingFor === u}
                >
                    <div class="import-unknown-name">{u}</div>
                    <div class="import-unknown-actions">
                        <button
                            class="range-pill"
                            class:range-pill--active={res.action === "create"}
                            onclick={() =>
                                setBodyResolution(u, { action: "create" })}
                        >
                            Create new
                        </button>
                        <button
                            class="range-pill"
                            class:range-pill--active={res.action === "map" ||
                                bodyPickingFor === u}
                            onclick={() => startBodyPicking(u)}
                        >
                            Map to existing
                        </button>
                        <button
                            class="range-pill"
                            class:range-pill--active={res.action === "skip"}
                            onclick={() =>
                                setBodyResolution(u, { action: "skip" })}
                        >
                            Skip
                        </button>
                        {#if res.action === "map"}
                            <span class="import-mapped-label">→ {res.name}</span
                            >
                        {/if}
                    </div>
                    {#if bodyPickingFor === u}
                        <div class="import-picker">
                            <p class="import-unknown-meta">Select metric</p>
                            <div class="list">
                                {#each bodyMetrics.filter((m) => !m.is_derived) as m}
                                    <button
                                        class="list-item"
                                        onclick={() =>
                                            bodyPickerSelectMetric(m)}
                                    >
                                        {m.name}
                                        <span class="import-unknown-meta"
                                            >{m.unit}</span
                                        >
                                    </button>
                                {/each}
                            </div>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
        <button
            class="add-btn"
            onclick={continueBodyToConfirm}
            style="display:flex;align-items:center;justify-content:center;gap:0.4rem;"
            >Continue <ChevronRight size={16} strokeWidth={1.5} /></button
        >
        <button class="delete-btn" onclick={reset}>Cancel</button>
    {:else if phase.name === "body_confirming"}
        <div class="import-summary">
            Import <strong>{phase.rowCount} measurements</strong> across
            <strong>{phase.dateCount} dates</strong>?
        </div>
        <button class="add-btn" onclick={confirmBodyImport}
            >Confirm import</button
        >
        <button class="delete-btn" onclick={reset}>Cancel</button>
    {:else if phase.name === "body_importing"}
        <p class="empty">Importing…</p>
    {:else if phase.name === "body_done"}
        <div class="import-summary">
            Imported
            <strong>{phase.result.measurements_imported} measurements</strong>
            across
            <strong>{phase.result.days_touched} days</strong>.
        </div>
        <button class="add-btn" onclick={reset}>Import another</button>
    {/if}
</div>
