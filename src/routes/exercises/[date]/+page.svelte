<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import PlusIcon from "$lib/icons/PlusIcon.svelte";

    import type { Category, Exercise } from "$lib/exercise";

    const date = $derived(page.params.date ?? "");

    type View =
        | { name: "categories" }
        | { name: "exercisesInCategory"; category: Category };
    let view = $state<View>({ name: "categories" });

    type MenuState =
        | { type: "closed" }
        | { type: "menu"; kind: "category" | "exercise"; id: number }
        | { type: "rename"; kind: "category" | "exercise"; id: number };
    let menuState = $state<MenuState>({ type: "closed" });

    type MergePrompt = {
        fromId: number;
        fromName: String;
        toName: String;
    } | null;
    let mergePrompt = $state<MergePrompt>(null);

    let categories = $state<Category[]>([]);
    let selectedCategory = $state<Category | undefined>(undefined);
    let exercisesInCategory = $state<Exercise[]>([]);

    onMount(async () => {
        categories = await invoke("list_exercise_categories");
    });

    async function selectCategory(category: Category) {
        selectedCategory = category;
        exercisesInCategory = await invoke("list_exercises_in_category", {
            categoryId: category.id,
        });
        view = "exercisesInCategory";
    }

    function selectExercise(exercise: Exercise) {
        goto(`/exercise/${exercise.id}/${date}`, { replaceState: true });
    }
</script>

<div class="page" role="presentation">
    <div onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
        {#if view.name === "categories"}
            <div class="header">
                <button
                    class="back-btn"
                    onclick={() => goto(date ? `/?date=${date}` : "/")}
                    >←</button
                >
                <h1>Select category</h1>
                <a class="new-btn" aria_label="Add new category">
                    <PlusIcon />
                </a>
            </div>
            <div class="list">
                {#each categories as category}
                    <button
                        class="list-item"
                        onclick={() => selectCategory(category)}
                    >
                        {category.name}
                    </button>
                {/each}
            </div>
        {:else}
            <div class="header">
                <button class="back-btn" onclick={() => (view = "categories")}
                    >←</button
                >
                <h1>{selectedCategory?.name}</h1>
            </div>
            {#if exercisesInCategory.length === 0}
                <p class="empty">No exercises in this category.</p>
            {:else}
                <div class="list">
                    {#each exercisesInCategory as exercise}
                        <button
                            class="list-item"
                            onclick={() => selectExercise(exercise)}
                        >
                            {exercise.name}
                        </button>
                    {/each}
                </div>
            {/if}
        {/if}
    </div>
</div>
