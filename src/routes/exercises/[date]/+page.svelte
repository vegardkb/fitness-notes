<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import PlusIcon from "$lib/icons/PlusIcon.svelte";
    import SelectList from "$lib/SelectList.svelte";
    import { toast } from "$lib/toast";

    import type { NamedId } from "$lib/exercise";

    const date = $derived(page.params.date ?? "");

    type View =
        | { name: "categories" }
        | { name: "exercisesInCategory"; category: NamedId };
    let view = $state<View>({ name: "categories" });

    let categories = $state<NamedId[]>([]);
    let exercisesInCategory = $state<NamedId[]>([]);
    let creating = $state(false);

    let mergeDialog = $state<HTMLDialogElement | null>(null);
    let mergeFrom = $state<NamedId | null>(null);
    let mergeTo = $state<NamedId | null>(null);

    let mergeExerciseDialog = $state<HTMLDialogElement | null>(null);
    let mergeExerciseFrom = $state<NamedId | null>(null);
    let mergeExerciseTo = $state<NamedId | null>(null);

    onMount(async () => {
        categories = await invoke("list_exercise_categories");
    });

    async function selectCategory(category: NamedId) {
        exercisesInCategory = await invoke("list_exercises_in_category", {
            categoryId: category.id,
        });
        view = { name: "exercisesInCategory", category };
        creating = false;
    }

    async function createCategory(name: string) {
        try {
            await invoke("create_category", { name });
            categories = await invoke("list_exercise_categories");
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function renameCategory(id: number, name: string) {
        try {
            await invoke("rename_category", { id, name });
            categories = await invoke("list_exercise_categories");
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function deleteCategory(id: number) {
        try {
            await invoke("delete_category", { id });
            categories = await invoke("list_exercise_categories");
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    function openMergeDialog(id: number) {
        const cat = categories.find((c) => c.id === id);
        if (!cat) return;
        mergeFrom = cat;
        mergeTo = null;
        mergeDialog?.showModal();
    }

    async function confirmMerge() {
        if (!mergeFrom || !mergeTo) return;
        try {
            await invoke("merge_category_into_existing", {
                fromId: mergeFrom.id,
                toId: mergeTo.id,
            });
            categories = await invoke("list_exercise_categories");
            mergeDialog?.close();
            toast.show(`Merged into "${mergeTo.name}"`);
            mergeFrom = null;
            mergeTo = null;
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function createExercise(name: string) {
        if (view.name !== "exercisesInCategory") return;
        try {
            await invoke("create_exercise", {
                name,
                categoryId: view.category.id,
            });
            exercisesInCategory = await invoke("list_exercises_in_category", {
                categoryId: view.category.id,
            });
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function renameExercise(id: number, name: string) {
        if (view.name !== "exercisesInCategory") return;
        try {
            await invoke("rename_exercise", { id, name });
            exercisesInCategory = await invoke("list_exercises_in_category", {
                categoryId: view.category.id,
            });
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function deleteExercise(id: number) {
        if (view.name !== "exercisesInCategory") return;
        try {
            await invoke("delete_exercise", { id });
            exercisesInCategory = await invoke("list_exercises_in_category", {
                categoryId: view.category.id,
            });
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    function openExerciseMergeDialog(id: number) {
        const ex = exercisesInCategory.find((e) => e.id === id);
        if (!ex) return;
        mergeExerciseFrom = ex;
        mergeExerciseTo = null;
        mergeExerciseDialog?.showModal();
    }

    async function confirmExerciseMerge() {
        if (
            !mergeExerciseFrom ||
            !mergeExerciseTo ||
            view.name !== "exercisesInCategory"
        )
            return;
        try {
            await invoke("merge_exercise_into_existing", {
                fromId: mergeExerciseFrom.id,
                toId: mergeExerciseTo.id,
            });
            exercisesInCategory = await invoke("list_exercises_in_category", {
                categoryId: view.category.id,
            });
            mergeExerciseDialog?.close();
            toast.show(`Merged into "${mergeExerciseTo.name}"`);
            mergeExerciseFrom = null;
            mergeExerciseTo = null;
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function selectExercise(exercise: NamedId) {
        let workout_exercise_id = await invoke<number>(
            "add_exercise_to_workout",
            {
                date: date,
                exerciseId: exercise.id,
            },
        );
        goto(`/exercise/${exercise.id}/${workout_exercise_id}`, {
            replaceState: true,
        });
    }

    const categoryActions = [
        { label: "Merge into…", action: openMergeDialog },
        { label: "Delete", danger: true, action: deleteCategory },
    ];

    const exerciseActions = [
        { label: "Merge into…", action: openExerciseMergeDialog },
        { label: "Delete", danger: true, action: deleteExercise },
    ];
</script>

<div class="page" role="presentation">
    {#if view.name === "categories"}
        <div class="header">
            <button
                class="back-btn"
                onclick={() => goto(date ? `/?date=${date}` : "/")}>←</button
            >
            <h1>Select category</h1>
            <button class="back-btn" onclick={() => (creating = true)}>
                <PlusIcon />
            </button>
        </div>
        <SelectList
            items={categories}
            placeholder="New category"
            bind:creating
            onselect={selectCategory}
            oncreate={createCategory}
            onrename={renameCategory}
            extraActions={categoryActions}
        />
    {:else if view.name === "exercisesInCategory"}
        <div class="header">
            <button
                class="back-btn"
                onclick={() => {
                    view = { name: "categories" };
                    creating = false;
                }}>←</button
            >
            <h1>{view.category.name}</h1>
            <button class="back-btn" onclick={() => (creating = true)}>
                <PlusIcon />
            </button>
        </div>
        {#if exercisesInCategory.length === 0 && !creating}
            <p class="empty">No exercises in this category.</p>
        {/if}
        <SelectList
            items={exercisesInCategory}
            placeholder="New exercise"
            bind:creating
            onselect={selectExercise}
            oncreate={createExercise}
            onrename={renameExercise}
            extraActions={exerciseActions}
        />
    {/if}
</div>

<dialog bind:this={mergeDialog}>
    <h2 class="dialog-title">Merge "{mergeFrom?.name}" into…</h2>
    <div class="list dialog-list">
        {#each categories.filter((c) => c.id !== mergeFrom?.id) as cat}
            <div
                class="list-item"
                class:list-item--selected={mergeTo?.id === cat.id}
                role="button"
                tabindex="0"
                onclick={() => (mergeTo = cat)}
                onkeydown={(e) => e.key === "Enter" && (mergeTo = cat)}
            >
                {cat.name}
            </div>
        {/each}
    </div>
    <div class="form-row">
        <button class="update-btn" disabled={!mergeTo} onclick={confirmMerge}
            >Merge</button
        >
        <button class="delete-btn" onclick={() => mergeDialog?.close()}
            >Cancel</button
        >
    </div>
</dialog>

<dialog bind:this={mergeExerciseDialog}>
    <h2 class="dialog-title">Merge "{mergeExerciseFrom?.name}" into…</h2>
    <div class="list dialog-list">
        {#each exercisesInCategory.filter((e) => e.id !== mergeExerciseFrom?.id) as ex}
            <div
                class="list-item"
                class:list-item--selected={mergeExerciseTo?.id === ex.id}
                role="button"
                tabindex="0"
                onclick={() => (mergeExerciseTo = ex)}
                onkeydown={(e) => e.key === "Enter" && (mergeExerciseTo = ex)}
            >
                {ex.name}
            </div>
        {/each}
    </div>
    <div class="form-row">
        <button
            class="update-btn"
            disabled={!mergeExerciseTo}
            onclick={confirmExerciseMerge}>Merge</button
        >
        <button class="delete-btn" onclick={() => mergeExerciseDialog?.close()}
            >Cancel</button
        >
    </div>
</dialog>
