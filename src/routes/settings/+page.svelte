<script lang="ts">
    import type { Category } from "$lib/exercise";
    import { invoke } from "@tauri-apps/api/core";
    import { ArrowLeft, ChevronRight } from "lucide-svelte";

    type ParsedRow = {
        date: string;
        exercise_name: string;
        category_name: string;
        weight_kg: number;
        reps: number;
        exercise_id: number | null;
    };
    type UnknownExercise = { csv_name: string; csv_category: string };
    type ParseResult = {
        rows: ParsedRow[];
        unknown_exercises: UnknownExercise[];
    };
    type Resolution =
        | { action: "create" }
        | { action: "map"; exercise_id: number; exercise_name: string };
    type ResolvedRow = {
        date: string;
        exercise_id: number | null;
        exercise_name: string;
        category_name: string;
        weight_kg: number;
        reps: number;
    };
    type ImportResult = { sets_imported: number; workouts_touched: number };
    type Exercise = { id: number; name: string };

    type Phase =
        | { name: "idle" }
        | {
              name: "resolving";
              rows: ParsedRow[];
              unknowns: UnknownExercise[];
              resolutions: Record<string, Resolution>;
          }
        | {
              name: "confirming";
              resolvedRows: ResolvedRow[];
              rowCount: number;
              dateCount: number;
          }
        | { name: "importing" }
        | { name: "done"; result: ImportResult }
        | { name: "error"; message: string };

    let phase = $state<Phase>({ name: "idle" });
    let fileInput = $state<HTMLInputElement>();

    let confirmingDelete = $state(false);

    async function deleteAllData() {
        await invoke("delete_all_data");
        confirmingDelete = false;
    }

    // Inline exercise picker state
    let pickingFor = $state<string | null>(null);
    let pickerView = $state<"categories" | "exercises">("categories");
    let pickerCategories = $state<Category[]>([]);
    let pickerExercises = $state<Exercise[]>([]);

    function buildResolvedRows(
        rows: ParsedRow[],
        resolutions: Record<string, Resolution>,
    ): ResolvedRow[] {
        return rows.map((row) => {
            if (row.exercise_id !== null) {
                return {
                    date: row.date,
                    exercise_id: row.exercise_id,
                    exercise_name: row.exercise_name,
                    category_name: row.category_name,
                    weight_kg: row.weight_kg,
                    reps: row.reps,
                };
            }
            const res = resolutions[row.exercise_name];
            if (res?.action === "map") {
                return {
                    date: row.date,
                    exercise_id: res.exercise_id,
                    exercise_name: res.exercise_name,
                    category_name: "",
                    weight_kg: row.weight_kg,
                    reps: row.reps,
                };
            }
            return {
                date: row.date,
                exercise_id: null,
                exercise_name: row.exercise_name,
                category_name: row.category_name,
                weight_kg: row.weight_kg,
                reps: row.reps,
            };
        });
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
    }

    async function startPicking(csvName: string) {
        if (pickingFor === csvName) {
            pickingFor = null;
            return;
        }
        pickingFor = csvName;
        pickerView = "categories";
        pickerCategories = await invoke<Category[]>("list_exercise_categories");
        pickerExercises = [];
    }

    async function pickerSelectCategory(cat: Category) {
        pickerView = "exercises";
        pickerExercises = await invoke<Exercise[]>(
            "list_exercises_in_category",
            {
                categoryId: cat.id,
            },
        );
    }

    function pickerSelectExercise(ex: Exercise) {
        if (phase.name !== "resolving" || !pickingFor) return;
        phase.resolutions[pickingFor] = {
            action: "map",
            exercise_id: ex.id,
            exercise_name: ex.name,
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
</script>

<div class="page">
    <div class="history-header">
        <a class="back-btn" href="/"><ArrowLeft size={18} strokeWidth={1.5} /></a>
        <h1>Settings</h1>
    </div>

    {#if phase.name === "idle"}
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
        </div>
        <input
            type="file"
            accept=".csv,text/csv,text/comma-separated-values,application/csv,application/vnd.ms-excel,text/plain"
            bind:this={fileInput}
            onchange={handleFile}
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
                            <span class="import-mapped-label"
                                >→ {res.exercise_name}</span
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
                                        ><ArrowLeft size={18} strokeWidth={1.5} /></button
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
        <button class="add-btn" onclick={continueToConfirm} style="display:flex;align-items:center;justify-content:center;gap:0.4rem;">Continue <ChevronRight size={16} strokeWidth={1.5} /></button>
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
    {/if}
</div>
