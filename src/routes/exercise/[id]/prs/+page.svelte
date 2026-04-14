<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { RepMax } from "$lib/exercise";
    import { formatWeight } from "$lib/exercise";

    const exerciseId = $derived(Number(page.params.id ?? "0"));

    let repMaxes = $state<RepMax[]>([]);

    onMount(async () => {
        repMaxes = await invoke<RepMax[]>("get_rep_maxes", { exerciseId });
    });
    const filled = $derived(() => {
        const byReps = new Map(repMaxes.map((r) => [r.reps, r]));
        const n = Math.max(...repMaxes.map((r) => r.reps), 0);
        return Array.from({ length: n }, (_, i) => {
            const k = i + 1;
            if (byReps.has(k)) return { ...byReps.get(k)!, ghost: false };
            // find closest l > k
            for (let l = k + 1; l <= n; l++) {
                if (byReps.has(l))
                    return { ...byReps.get(l)!, ghost: true, reps: k };
            }
            return { ...byReps.get(n)!, ghost: true, reps: k };
        }).filter(Boolean);
    });
    let repMaxesFilled = $derived(filled());
</script>

<div class="body">
    {#if repMaxes.length === 0}
        <p class="empty">No PRs for this exercise.</p>
    {:else}
        <table class="pr-table">
            <thead>
                <tr>
                    <th class="th-rm">PRs</th>
                </tr>
            </thead>
            <tbody>
                {#each repMaxesFilled as item}
                    <tr class:ghost={item.ghost}>
                        <td class="td-reps">{item.reps}</td>
                        <td class="td-rm">RM</td>
                        <td class="td-weight">
                            <a href={`/exercise/${exerciseId}/${item.date}`}>
                                <span class="stat-val stat-val--weight"
                                    >{formatWeight(item.weight_kg)}<span
                                        class="stat-unit"
                                    >
                                        kg</span
                                    ></span
                                >
                                <span class="date"
                                    >{new Intl.DateTimeFormat("no", {
                                        dateStyle: "short",
                                    }).format(
                                        new Date(item.date + "T00:00:00"),
                                    )}</span
                                >
                            </a>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>
