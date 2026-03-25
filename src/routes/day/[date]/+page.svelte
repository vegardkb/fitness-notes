<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";

    type Set = {
        id: number;
        set_order: number;
        weight_kg: number;
        reps: number;
        notes: string | null;
        was_pr_at_time: boolean;
        is_current_pr: boolean;
    };

    type ExerciseWithSets = {
        exercise_id: number;
        exercise_name: string;
        category: string;
        exercise_order: number;
        sets: Set[];
    };

    type Exercise = {
        id: number;
        name: string;
    };

    // Parse "YYYY-MM-DD" as a local date (not UTC)
    function parseLocal(dateStr: string): Date {
        const [y, m, d] = dateStr.split("-").map(Number);
        return new Date(y, m - 1, d);
    }

    function toDateStr(d: Date): string {
        const y = d.getFullYear();
        const m = String(d.getMonth() + 1).padStart(2, "0");
        const day = String(d.getDate()).padStart(2, "0");
        return `${y}-${m}-${day}`;
    }

    function todayStr(): string {
        return toDateStr(new Date());
    }

    function formatDate(dateStr: string): string {
        const today = todayStr();
        const yesterday = toDateStr(
            new Date(new Date().setDate(new Date().getDate() - 1)),
        );
        const tomorrow = toDateStr(
            new Date(new Date().setDate(new Date().getDate() + 1)),
        );
        if (dateStr === today) return "Today";
        if (dateStr === yesterday) return "Yesterday";
        if (dateStr === tomorrow) return "Tomorrow";
        return parseLocal(dateStr).toLocaleDateString("en-US", {
            weekday: "long",
            month: "long",
            day: "numeric",
        });
    }

    function offsetDay(dateStr: string, delta: number): string {
        const d = parseLocal(dateStr);
        d.setDate(d.getDate() + delta);
        return toDateStr(d);
    }

    const date = $derived(page.params.date ?? '');
    const label = $derived(formatDate(date));

    let exercises = $state<ExerciseWithSets[]>([]);
    let view = $state<"exercises" | "categories" | "exercisesInCategory">(
        "exercises",
    );
    let categories = $state<string[]>([]);
    let selectedCategory = $state("");
    let exercisesInCategory = $state<Exercise[]>([]);

    $effect(() => {
        const d = date;
        view = "exercises";
        invoke<ExerciseWithSets[]>("get_workout_for_date", { date: d }).then(
            (result) => {
                exercises = result;
            },
        );
    });

    async function showCategories() {
        if (categories.length === 0) {
            categories = await invoke("list_exercise_categories");
        }
        view = "categories";
    }

    async function selectCategory(category: string) {
        selectedCategory = category;
        exercisesInCategory = await invoke("list_exercises_in_category", {
            category,
        });
        view = "exercisesInCategory";
    }

    function selectExercise(exercise: Exercise) {
        goto(`/exercise/${exercise.id}/${date}`);
    }

    function navigateDay(delta: number) {
        goto(`/day/${offsetDay(date, delta)}`);
    }

    // Swipe gesture
    let touchStartX = 0;

    function handleTouchStart(e: TouchEvent) {
        touchStartX = e.touches[0].clientX;
    }

    function handleTouchEnd(e: TouchEvent) {
        const delta = e.changedTouches[0].clientX - touchStartX;
        if (Math.abs(delta) > 50) {
            navigateDay(delta > 0 ? -1 : 1);
        }
    }
</script>

<main class="page" ontouchstart={handleTouchStart} ontouchend={handleTouchEnd}>
    {#if view === "exercises"}
        <div class="header">
            <a class="back-btn" href="/">←</a>
            <div class="date-nav">
                <button class="date-nav-btn" onclick={() => navigateDay(-1)}
                    >‹</button
                >
                <button
                    class="date-label"
                    onclick={() => goto(`/day/${todayStr()}`)}>{label}</button
                >
                <button class="date-nav-btn" onclick={() => navigateDay(1)}
                    >›</button
                >
            </div>
        </div>

        {#if exercises.length === 0}
            <p class="empty">No exercises logged. Add one below.</p>
        {:else}
            <div class="list">
                {#each exercises as ex}
                    <div class="exercise-card">
                        <button
                            class="exercise-card-header"
                            onclick={() =>
                                goto(`/exercise/${ex.exercise_id}/${date}`)}
                        >
                            <span>{ex.exercise_name}</span>
                            <span class="muted">→</span>
                        </button>
                        <div class="exercise-card-sets">
                            {#each ex.sets as set, i}
                                <div class="set-row">
                                    <span class="set-label">{i + 1}</span>
                                    <span class="set-weight">{set.weight_kg} kg × {set.reps}</span>
                                    <span class="set-badge">
                                        {#if set.is_current_pr}
                                            <span class="pr-badge pr-badge--current">PR</span>
                                        {:else if set.was_pr_at_time}
                                            <span class="pr-badge pr-badge--historic">PR</span>
                                        {/if}
                                    </span>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}

        <button class="add-btn" onclick={showCategories}>+ Add exercise</button>
    {:else if view === "categories"}
        <div class="header">
            <button class="back-btn" onclick={() => (view = "exercises")}
                >←</button
            >
            <h1>Select category</h1>
        </div>

        <div class="list">
            {#each categories as category}
                <button
                    class="list-item"
                    onclick={() => selectCategory(category)}
                >
                    {category}
                </button>
            {/each}
        </div>
    {:else if view === "exercisesInCategory"}
        <div class="header">
            <button class="back-btn" onclick={() => (view = "categories")}
                >←</button
            >
            <h1>{selectedCategory}</h1>
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
</main>
