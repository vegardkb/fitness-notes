<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    type Exercise = {
        id: number;
        name: string;
    };

    let { date, onclose }: { date: string; onclose: () => void } = $props();

    let view = $state<"categories" | "exercisesInCategory">("categories");
    let categories = $state<string[]>([]);
    let selectedCategory = $state("");
    let exercisesInCategory = $state<Exercise[]>([]);

    onMount(async () => {
        categories = await invoke("list_exercise_categories");
    });

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") onclose();
    }

    async function selectCategory(category: string) {
        selectedCategory = category;
        exercisesInCategory = await invoke("list_exercises_in_category", { category });
        view = "exercisesInCategory";
    }

    function selectExercise(exercise: Exercise) {
        goto(`/exercise/${exercise.id}/${date}`);
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="modal-backdrop" onclick={onclose} onkeydown={handleKeydown} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
        {#if view === "categories"}
            <div class="header">
                <button class="back-btn" onclick={onclose}>←</button>
                <h1>Select category</h1>
            </div>
            <div class="list">
                {#each categories as category}
                    <button class="list-item" onclick={() => selectCategory(category)}>
                        {category}
                    </button>
                {/each}
            </div>
        {:else}
            <div class="header">
                <button class="back-btn" onclick={() => (view = "categories")}>←</button>
                <h1>{selectedCategory}</h1>
            </div>
            {#if exercisesInCategory.length === 0}
                <p class="empty">No exercises in this category.</p>
            {:else}
                <div class="list">
                    {#each exercisesInCategory as exercise}
                        <button class="list-item" onclick={() => selectExercise(exercise)}>
                            {exercise.name}
                        </button>
                    {/each}
                </div>
            {/if}
        {/if}
    </div>
</div>
