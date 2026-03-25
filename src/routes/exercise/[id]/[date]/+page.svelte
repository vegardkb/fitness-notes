<script lang="ts">
  import { page } from '$app/state';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

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
    sets: Set[];
  };

  type Exercise = {
    id: number;
    name: string;
  };

  const exerciseId = $derived(Number(page.params.id));
  const date = $derived(page.params.date);

  let exerciseName = $state('');
  let sets = $state<Set[]>([]);
  let weightInput = $state('');
  let repsInput = $state('');
  let adding = $state(false);

  function defaultToLastSet(setList: Set[]) {
    const last = setList.at(-1);
    if (last) {
      weightInput = String(last.weight_kg);
      repsInput = String(last.reps);
    }
  }

  onMount(async () => {
    const [workout, exercise] = await Promise.all([
      invoke<ExerciseWithSets[]>('get_workout_for_date', { date }),
      invoke<Exercise>('get_exercise', { id: exerciseId }),
    ]);
    exerciseName = exercise.name;
    const found = workout.find((e) => e.exercise_id === exerciseId);
    sets = found?.sets ?? [];
    defaultToLastSet(sets);
  });

  async function addSet() {
    if (!weightInput || !repsInput) return;
    adding = true;
    try {
      const newSet = await invoke<Set>('add_set', {
        date,
        exerciseId,
        weightKg: parseFloat(weightInput),
        reps: parseInt(repsInput),
        notes: null,
      });
      sets.push(newSet);
      defaultToLastSet(sets);
    } finally {
      adding = false;
    }
  }
</script>

<div class="page">
  <div class="header">
    <a class="back-btn" href="/day/{date}">←</a>
    <h1>{exerciseName}</h1>
  </div>

  {#if sets.length === 0}
    <p class="empty">No sets yet. Add your first set below.</p>
  {:else}
    <div class="list">
      {#each sets as set}
        <div class="list-item">
          <span>{set.weight_kg} kg × {set.reps}</span>
          <div style="display: flex; gap: 0.5rem; align-items: center;">
            {#if set.is_current_pr}
              <span class="pr-badge pr-badge--current">PR</span>
            {:else if set.was_pr_at_time}
              <span class="pr-badge pr-badge--historic">PR</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="form">
    <div class="form-row">
      <div class="field">
        <label for="weight">Weight (kg)</label>
        <input
          id="weight"
          type="number"
          step="0.5"
          min="0"
          placeholder="100"
          bind:value={weightInput}
        />
      </div>
      <div class="field">
        <label for="reps">Reps</label>
        <input
          id="reps"
          type="number"
          min="1"
          placeholder="5"
          bind:value={repsInput}
        />
      </div>
    </div>
    <button class="add-btn" onclick={addSet} disabled={adding || !weightInput || !repsInput}>
      {adding ? 'Adding…' : 'Add set'}
    </button>
  </div>
</div>
