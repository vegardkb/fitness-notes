<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";

    const MONTH_NAMES = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    const WEEKDAYS = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

    function toDateStr(y: number, m: number, d: number): string {
        return `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    }

    function todayStr(): string {
        const n = new Date();
        return toDateStr(n.getFullYear(), n.getMonth() + 1, n.getDate());
    }

    const today = todayStr();
    const now = new Date();

    let year = $state(now.getFullYear());
    let month = $state(now.getMonth() + 1); // 1-indexed
    let activeDates = $state(new Set<string>());

    $effect(() => {
        const y = year;
        const m = month;
        invoke<string[]>("get_active_dates", { year: y, month: m }).then((dates) => {
            activeDates = new Set(dates);
        });
    });

    function prevMonth() {
        if (month === 1) { month = 12; year -= 1; }
        else { month -= 1; }
    }

    function nextMonth() {
        if (month === 12) { month = 1; year += 1; }
        else { month += 1; }
    }

    const cells = $derived.by(() => {
        const firstDow = new Date(year, month - 1, 1).getDay();
        const daysInMonth = new Date(year, month, 0).getDate();
        const result: Array<{ day: number; dateStr: string } | null> = [];
        for (let i = 0; i < firstDow; i++) result.push(null);
        for (let d = 1; d <= daysInMonth; d++) {
            result.push({ day: d, dateStr: toDateStr(year, month, d) });
        }
        return result;
    });
</script>

<main class="page">
    <div class="cal-header">
        <button class="date-nav-btn" onclick={prevMonth}>‹</button>
        <span class="cal-month-label">{MONTH_NAMES[month - 1]} {year}</span>
        <button class="date-nav-btn" onclick={nextMonth}>›</button>
    </div>

    <div class="cal-weekdays">
        {#each WEEKDAYS as day}
            <span>{day}</span>
        {/each}
    </div>

    <div class="cal-grid">
        {#each cells as cell}
            {#if cell === null}
                <div class="cal-day cal-day--empty"></div>
            {:else}
                <button
                    class="cal-day"
                    class:cal-day--today={cell.dateStr === today}
                    onclick={() => goto(`/day/${cell.dateStr}`)}
                >
                    <span>{cell.day}</span>
                    {#if activeDates.has(cell.dateStr)}
                        <span class="cal-dot"></span>
                    {/if}
                </button>
            {/if}
        {/each}
    </div>
</main>
