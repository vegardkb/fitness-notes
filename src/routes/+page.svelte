<script module lang="ts">
    // Module-level state — persists across client-side navigations
    let _savedDays: string[] = [];
    let _savedScrollY = 0;
    let _savedOldest = "";
    let _savedNewest = "";
</script>

<script lang="ts">
    import { onMount } from "svelte";
    import { Calendar, Settings } from "lucide-svelte";
    import DayCard from "$lib/DayCard.svelte";
    import { offsetDate, todayStr, isValidDate } from "$lib/date";

    let days = $state<string[]>([]);
    let loadingOlder = $state(false);
    let loadingNewer = $state(false);
    let oldestLoaded = "";
    let newestLoaded = "";

    let topSentinel: HTMLElement;
    let bottomSentinel: HTMLElement;
    let observer: IntersectionObserver | null = null;

    function minDate(a: string, b: string): string {
        return a < b ? a : b;
    }

    function datesBetween(from: string, to: string): string[] {
        const result: string[] = [];
        let cur = to;
        while (cur >= from) {
            result.push(cur);
            cur = offsetDate(cur, -1);
        }
        return result;
    }

    function loadRange(from: string, to: string) {
        const incoming = new Set(datesBetween(from, to));
        const existing = new Set(days);
        for (const d of existing) incoming.add(d);
        days = Array.from(incoming).sort().reverse();
        oldestLoaded = days.at(-1)!;
        newestLoaded = days[0];
        _savedDays = days;
        _savedOldest = oldestLoaded;
        _savedNewest = newestLoaded;
    }

    function scrollToDate(date: string) {
        document
            .getElementById(`day-${date}`)
            ?.scrollIntoView({ behavior: "instant", block: "center" });
    }

    function setupObserver() {
        observer?.disconnect();
        observer = new IntersectionObserver(
            async (entries) => {
                for (const entry of entries) {
                    if (!entry.isIntersecting) continue;

                    if (entry.target === bottomSentinel && !loadingOlder) {
                        loadingOlder = true;
                        loadRange(
                            offsetDate(oldestLoaded, -14),
                            offsetDate(oldestLoaded, -1),
                        );
                        loadingOlder = false;
                    }

                    if (entry.target === topSentinel && !loadingNewer) {
                        const today = todayStr();
                        if (newestLoaded >= today) continue;
                        loadingNewer = true;
                        const from = offsetDate(newestLoaded, 1);
                        const to = minDate(offsetDate(from, 13), today);
                        const prevHeight =
                            document.documentElement.scrollHeight;
                        loadRange(from, to);
                        await new Promise<void>((r) =>
                            requestAnimationFrame(() => {
                                window.scrollBy(
                                    0,
                                    document.documentElement.scrollHeight -
                                        prevHeight,
                                );
                                r();
                            }),
                        );
                        loadingNewer = false;
                    }
                }
            },
            { rootMargin: "400px" },
        );
        observer.observe(topSentinel);
        observer.observe(bottomSentinel);
    }

    onMount(() => {
        const targetDate = new URLSearchParams(location.search).get("date");

        if (_savedDays.length > 0) {
            // Back-navigation: restore saved state
            days = _savedDays;
            oldestLoaded = _savedOldest;
            newestLoaded = _savedNewest;
            requestAnimationFrame(() => {
                if (targetDate && isValidDate(targetDate)) {
                    if (!_savedDays.includes(targetDate)) {
                        loadRange(
                            offsetDate(targetDate, -3),
                            minDate(offsetDate(targetDate, 3), todayStr()),
                        );
                    }
                    scrollToDate(targetDate);
                } else {
                    window.scrollTo(0, _savedScrollY);
                }
                setTimeout(setupObserver, 150);
            });
        } else {
            // Fresh load or deep-link
            const anchor =
                targetDate && isValidDate(targetDate) ? targetDate : todayStr();
            loadRange(
                offsetDate(anchor, -13),
                minDate(offsetDate(anchor, 3), todayStr()),
            );
            requestAnimationFrame(() => {
                if (targetDate) scrollToDate(targetDate);
                setupObserver();
            });
        }

        return () => {
            _savedScrollY = window.scrollY;
            observer?.disconnect();
        };
    });
</script>

<div class="page">
    <div class="app-header-wrap">
        <header class="app-header">
            <span class="app-title">Fitness Notes</span>
            <div class="app-header-icons">
                <a href="/calendar" class="back-btn" aria-label="Calendar">
                    <Calendar size={20} strokeWidth={1.5} />
                </a>
                <a href="/settings" class="back-btn" aria-label="Settings">
                    <Settings size={20} strokeWidth={1.5} />
                </a>
            </div>
        </header>
    </div>

    <div class="feed">
        <div bind:this={topSentinel} class="feed-sentinel"></div>
        {#each days as date (date)}
            <DayCard {date} />
        {/each}
        <div bind:this={bottomSentinel} class="feed-sentinel"></div>
        {#if loadingOlder}
            <p class="feed-loading">Loading…</p>
        {/if}
    </div>
</div>
